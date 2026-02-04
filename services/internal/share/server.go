package share

import (
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/zeebo/blake3"

	"latticefs/services/internal/ipc"
	pb "latticefs/services/proto"

	"google.golang.org/protobuf/proto"
)

type Server struct{}

func NewServer() *Server {
	return &Server{}
}

type shareRequest struct {
	ObjectID    string `json:"object_id"`
	AudienceDID string `json:"audience_did"`
	Capability  string `json:"capability"`
	ExpiresIn   int64  `json:"expires_in"`
}

type shareResponse struct {
	UCANToken string `json:"ucan_token"`
	ExpiresAt string `json:"expires_at"`
}

type revokeRequest struct {
	UCANCID string `json:"ucan_cid"`
	Reason  string `json:"reason"`
}

type revokeResponse struct {
	Revoked   bool   `json:"revoked"`
	RevokedAt string `json:"revoked_at"`
}

func (s *Server) Register(mux *http.ServeMux) {
	mux.HandleFunc("/share", s.handleShare)
	mux.HandleFunc("/revoke", s.handleRevoke)
	mux.HandleFunc("/fetch/", s.handleFetch)
}

func (s *Server) handleShare(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	token, err := bearerToken(r)
	if err != nil {
		http.Error(w, "Unauthorized", http.StatusUnauthorized)
		return
	}

	revocations, _ := LoadRevocations()
	ucan, err := ValidateUcan(token, revocations)
	if err != nil {
		http.Error(w, "Unauthorized", http.StatusUnauthorized)
		return
	}

	var req shareRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "Invalid request", http.StatusBadRequest)
		return
	}

	if req.ObjectID == "" || req.AudienceDID == "" || req.Capability == "" {
		http.Error(w, "Missing fields", http.StatusBadRequest)
		return
	}
	if !IsValidPermission(req.Capability) {
		http.Error(w, "Invalid capability", http.StatusBadRequest)
		return
	}
	if req.ExpiresIn <= 0 {
		http.Error(w, "expires_in must be a positive integer", http.StatusBadRequest)
		return
	}

	objectID, err := uuid.Parse(req.ObjectID)
	if err != nil {
		http.Error(w, "Invalid object id", http.StatusBadRequest)
		return
	}
	resource := fmt.Sprintf("latticefs:object:%s", objectID.String())

	if !HasPermission(ucan, resource, "share") && !HasPermission(ucan, resource, "admin") {
		http.Error(w, "Forbidden", http.StatusForbidden)
		return
	}
	if !HasPermission(ucan, resource, req.Capability) {
		http.Error(w, "Forbidden", http.StatusForbidden)
		return
	}

	expiresAt := time.Now().Add(time.Duration(req.ExpiresIn) * time.Second)

	ipcReq := &pb.ShareRequest{
		ObjectId:    &pb.ObjectID{Uuid: objectID[:]},
		AudienceDid: req.AudienceDID,
		Capability:  req.Capability,
		ExpiresAt:   &pb.Timestamp{Micros: expiresAt.UnixMicro()},
		Facts:       map[string]string{},
	}

	conn, err := ipc.Connect()
	if err != nil {
		http.Error(w, "IPC unavailable", http.StatusServiceUnavailable)
		return
	}
	defer conn.Close()

	if err := ipc.SendMessage(conn, ipc.MsgShareRequest, ipcReq); err != nil {
		http.Error(w, "IPC error", http.StatusServiceUnavailable)
		return
	}

	msgType, payload, err := ipc.RecvMessage(conn)
	if err != nil || msgType != ipc.MsgShareResponse {
		http.Error(w, "IPC error", http.StatusServiceUnavailable)
		return
	}

	var resp pb.ShareResponse
	if err := proto.Unmarshal(payload, &resp); err != nil {
		http.Error(w, "IPC error", http.StatusServiceUnavailable)
		return
	}

	switch result := resp.Result.(type) {
	case *pb.ShareResponse_UcanToken:
		writeJSON(w, shareResponse{
			UCANToken: result.UcanToken,
			ExpiresAt: expiresAt.UTC().Format(time.RFC3339),
		})
	case *pb.ShareResponse_Error:
		http.Error(w, result.Error.Message, http.StatusBadRequest)
	default:
		http.Error(w, "IPC error", http.StatusServiceUnavailable)
	}
}

func (s *Server) handleFetch(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	objectIDStr := strings.TrimPrefix(r.URL.Path, "/fetch/")
	if objectIDStr == "" {
		http.Error(w, "Missing object id", http.StatusBadRequest)
		return
	}
	objectID, err := uuid.Parse(objectIDStr)
	if err != nil {
		http.Error(w, "Invalid object id", http.StatusBadRequest)
		return
	}

	token, err := bearerToken(r)
	if err != nil {
		http.Error(w, "Unauthorized", http.StatusUnauthorized)
		return
	}

	revocations, _ := LoadRevocations()
	ucan, err := ValidateUcan(token, revocations)
	if err != nil {
		http.Error(w, "Unauthorized", http.StatusUnauthorized)
		return
	}
	resource := fmt.Sprintf("latticefs:object:%s", objectID.String())
	if !HasPermission(ucan, resource, "read") {
		http.Error(w, "Forbidden", http.StatusForbidden)
		return
	}

	conn, err := ipc.Connect()
	if err != nil {
		http.Error(w, "IPC unavailable", http.StatusServiceUnavailable)
		return
	}
	defer conn.Close()

	ipcReq := &pb.FetchRequest{
		ObjectId:  &pb.ObjectID{Uuid: objectID[:]},
		UcanToken: token,
	}

	if err := ipc.SendMessage(conn, ipc.MsgFetchRequest, ipcReq); err != nil {
		http.Error(w, "IPC error", http.StatusServiceUnavailable)
		return
	}

	msgType, payload, err := ipc.RecvMessage(conn)
	if err != nil || msgType != ipc.MsgFetchResponse {
		http.Error(w, "IPC error", http.StatusServiceUnavailable)
		return
	}

	var resp pb.FetchResponse
	if err := proto.Unmarshal(payload, &resp); err != nil {
		http.Error(w, "IPC error", http.StatusServiceUnavailable)
		return
	}

	switch result := resp.Result.(type) {
	case *pb.FetchResponse_Data:
		data := result.Data
		if data == nil {
			http.Error(w, "Not found", http.StatusNotFound)
			return
		}

		versionID := ""
		if data.VersionId != nil {
			if v, err := uuid.FromBytes(data.VersionId.Uuid); err == nil {
				versionID = v.String()
			}
		}

		hash := blake3.Sum256(data.Content)
		w.Header().Set("Content-Type", "application/octet-stream")
		if versionID != "" {
			w.Header().Set("X-NeuralFS-Version", versionID)
		}
		w.Header().Set("X-NeuralFS-Blake3", hex.EncodeToString(hash[:]))
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write(data.Content)
	case *pb.FetchResponse_Error:
		http.Error(w, result.Error.Message, http.StatusBadRequest)
	default:
		http.Error(w, "IPC error", http.StatusServiceUnavailable)
	}
}

func (s *Server) handleRevoke(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	token, err := bearerToken(r)
	if err != nil {
		http.Error(w, "Unauthorized", http.StatusUnauthorized)
		return
	}
	revocations, _ := LoadRevocations()
	if _, err := ValidateUcan(token, revocations); err != nil {
		http.Error(w, "Unauthorized", http.StatusUnauthorized)
		return
	}

	var req revokeRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "Invalid request", http.StatusBadRequest)
		return
	}
	if req.UCANCID == "" {
		http.Error(w, "Missing ucan_cid", http.StatusBadRequest)
		return
	}

	conn, err := ipc.Connect()
	if err != nil {
		http.Error(w, "IPC unavailable", http.StatusServiceUnavailable)
		return
	}
	defer conn.Close()

	ipcReq := &pb.RevokeRequest{
		UcanCid: req.UCANCID,
		Reason:  req.Reason,
	}

	if err := ipc.SendMessage(conn, ipc.MsgRevokeRequest, ipcReq); err != nil {
		http.Error(w, "IPC error", http.StatusServiceUnavailable)
		return
	}

	msgType, payload, err := ipc.RecvMessage(conn)
	if err != nil || msgType != ipc.MsgRevokeResponse {
		http.Error(w, "IPC error", http.StatusServiceUnavailable)
		return
	}

	var resp pb.RevokeResponse
	if err := proto.Unmarshal(payload, &resp); err != nil {
		http.Error(w, "IPC error", http.StatusServiceUnavailable)
		return
	}

	switch result := resp.Result.(type) {
	case *pb.RevokeResponse_Success:
		writeJSON(w, revokeResponse{
			Revoked:   result.Success,
			RevokedAt: time.Now().UTC().Format(time.RFC3339),
		})
	case *pb.RevokeResponse_Error:
		http.Error(w, result.Error.Message, http.StatusBadRequest)
	default:
		http.Error(w, "IPC error", http.StatusServiceUnavailable)
	}
}

func bearerToken(r *http.Request) (string, error) {
	auth := r.Header.Get("Authorization")
	if auth == "" {
		return "", errors.New("missing authorization")
	}
	if !strings.HasPrefix(auth, "Bearer ") {
		return "", errors.New("invalid authorization")
	}
	return strings.TrimPrefix(auth, "Bearer "), nil
}

func writeJSON(w http.ResponseWriter, payload any) {
	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(payload)
}
