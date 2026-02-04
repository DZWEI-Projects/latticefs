package api

import (
    "context"
    "errors"
    "fmt"
    "net"
    "strings"

    "github.com/google/uuid"
    "google.golang.org/grpc"
    "google.golang.org/grpc/codes"
    "google.golang.org/grpc/metadata"
    "google.golang.org/grpc/status"
    "google.golang.org/protobuf/proto"

    "latticefs/services/internal/ipc"
    "latticefs/services/internal/share"
    pb "latticefs/services/proto"
)

type GrpcServer struct {
    pb.UnimplementedShareServiceServer
}

func StartGrpcServer(port int) error {
    lis, err := net.Listen("tcp", fmt.Sprintf(":%d", port))
    if err != nil {
        return err
    }

    server := grpc.NewServer()
    pb.RegisterShareServiceServer(server, &GrpcServer{})

    go func() {
        _ = server.Serve(lis)
    }()

    return nil
}

func (s *GrpcServer) Share(ctx context.Context, req *pb.ShareRequest) (*pb.ShareResponse, error) {
    token, err := authFromContext(ctx)
    if err != nil {
        return nil, status.Error(codes.Unauthenticated, "missing authorization")
    }
    revocations, _ := share.LoadRevocations()
    ucan, err := share.ValidateUcan(token, revocations)
    if err != nil {
        return nil, status.Error(codes.Unauthenticated, "invalid authorization")
    }

    if req.ObjectId == nil {
        return nil, status.Error(codes.InvalidArgument, "missing object_id")
    }
    if !share.IsValidPermission(req.Capability) {
        return nil, status.Error(codes.InvalidArgument, "invalid capability")
    }

    objectID, err := uuid.FromBytes(req.ObjectId.Uuid)
    if err != nil {
        return nil, status.Error(codes.InvalidArgument, "invalid object_id")
    }
    resource := fmt.Sprintf("latticefs:object:%s", objectID.String())
    if !share.HasPermission(ucan, resource, "share") && !share.HasPermission(ucan, resource, "admin") {
        return nil, status.Error(codes.PermissionDenied, "forbidden")
    }
    if !share.HasPermission(ucan, resource, req.Capability) {
        return nil, status.Error(codes.PermissionDenied, "forbidden")
    }

    if req.ExpiresAt == nil {
        return nil, status.Error(codes.InvalidArgument, "missing expires_at")
    }

    conn, err := ipc.Connect()
    if err != nil {
        return nil, status.Error(codes.Unavailable, "ipc unavailable")
    }
    defer conn.Close()

    if err := ipc.SendMessage(conn, ipc.MsgShareRequest, req); err != nil {
        return nil, status.Error(codes.Unavailable, "ipc error")
    }

    msgType, payload, err := ipc.RecvMessage(conn)
    if err != nil || msgType != ipc.MsgShareResponse {
        return nil, status.Error(codes.Unavailable, "ipc error")
    }

    var resp pb.ShareResponse
    if err := proto.Unmarshal(payload, &resp); err != nil {
        return nil, status.Error(codes.Unavailable, "ipc error")
    }

    return &resp, nil
}

func (s *GrpcServer) Fetch(ctx context.Context, req *pb.FetchRequest) (*pb.FetchResponse, error) {
    token, err := authFromContext(ctx)
    if err != nil {
        return nil, status.Error(codes.Unauthenticated, "missing authorization")
    }
    revocations, _ := share.LoadRevocations()
    ucan, err := share.ValidateUcan(token, revocations)
    if err != nil {
        return nil, status.Error(codes.Unauthenticated, "invalid authorization")
    }

    if req.ObjectId == nil {
        return nil, status.Error(codes.InvalidArgument, "missing object_id")
    }
    objectID, err := uuid.FromBytes(req.ObjectId.Uuid)
    if err != nil {
        return nil, status.Error(codes.InvalidArgument, "invalid object_id")
    }
    resource := fmt.Sprintf("latticefs:object:%s", objectID.String())
    if !share.HasPermission(ucan, resource, "read") {
        return nil, status.Error(codes.PermissionDenied, "forbidden")
    }

    req.UcanToken = token

    conn, err := ipc.Connect()
    if err != nil {
        return nil, status.Error(codes.Unavailable, "ipc unavailable")
    }
    defer conn.Close()

    if err := ipc.SendMessage(conn, ipc.MsgFetchRequest, req); err != nil {
        return nil, status.Error(codes.Unavailable, "ipc error")
    }

    msgType, payload, err := ipc.RecvMessage(conn)
    if err != nil || msgType != ipc.MsgFetchResponse {
        return nil, status.Error(codes.Unavailable, "ipc error")
    }

    var resp pb.FetchResponse
    if err := proto.Unmarshal(payload, &resp); err != nil {
        return nil, status.Error(codes.Unavailable, "ipc error")
    }

    return &resp, nil
}

func (s *GrpcServer) Revoke(ctx context.Context, req *pb.RevokeRequest) (*pb.RevokeResponse, error) {
    token, err := authFromContext(ctx)
    if err != nil {
        return nil, status.Error(codes.Unauthenticated, "missing authorization")
    }
    revocations, _ := share.LoadRevocations()
    if _, err := share.ValidateUcan(token, revocations); err != nil {
        return nil, status.Error(codes.Unauthenticated, "invalid authorization")
    }

    conn, err := ipc.Connect()
    if err != nil {
        return nil, status.Error(codes.Unavailable, "ipc unavailable")
    }
    defer conn.Close()

    if err := ipc.SendMessage(conn, ipc.MsgRevokeRequest, req); err != nil {
        return nil, status.Error(codes.Unavailable, "ipc error")
    }

    msgType, payload, err := ipc.RecvMessage(conn)
    if err != nil || msgType != ipc.MsgRevokeResponse {
        return nil, status.Error(codes.Unavailable, "ipc error")
    }

    var resp pb.RevokeResponse
    if err := proto.Unmarshal(payload, &resp); err != nil {
        return nil, status.Error(codes.Unavailable, "ipc error")
    }

    return &resp, nil
}

func authFromContext(ctx context.Context) (string, error) {
    md, ok := metadata.FromIncomingContext(ctx)
    if !ok {
        return "", errors.New("missing metadata")
    }
    values := md.Get("authorization")
    if len(values) == 0 {
        return "", errors.New("missing authorization")
    }
    auth := values[0]
    if strings.HasPrefix(auth, "Bearer ") {
        return strings.TrimPrefix(auth, "Bearer "), nil
    }
    return auth, nil
}
