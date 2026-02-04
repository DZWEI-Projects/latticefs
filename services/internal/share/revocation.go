package share

import (
    "bufio"
    "crypto/ed25519"
    "encoding/json"
    "os"
    "path/filepath"
)

type Revocation struct {
    UCANCID   string  `json:"ucan_cid"`
    RevokedAt uint64  `json:"revoked_at"`
    RevokedBy string  `json:"revoked_by"`
    Reason    *string `json:"reason,omitempty"`
    Signature []uint8 `json:"signature"`
}

type revocationPayload struct {
    UCANCID   string  `json:"ucan_cid"`
    RevokedAt uint64  `json:"revoked_at"`
    RevokedBy string  `json:"revoked_by"`
    Reason    *string `json:"reason,omitempty"`
}

type RevocationIndex struct {
    items map[string]Revocation
}

func LoadRevocations() (*RevocationIndex, error) {
    path, err := revocationLogPath()
    if err != nil {
        return nil, err
    }

    file, err := os.Open(path)
    if err != nil {
        if os.IsNotExist(err) {
            return &RevocationIndex{items: map[string]Revocation{}}, nil
        }
        return nil, err
    }
    defer file.Close()

    index := &RevocationIndex{items: map[string]Revocation{}}
    scanner := bufio.NewScanner(file)
    for scanner.Scan() {
        line := scanner.Bytes()
        if len(line) == 0 {
            continue
        }
        var rev Revocation
        if err := json.Unmarshal(line, &rev); err != nil {
            continue
        }
        if verifyRevocation(rev) {
            index.items[rev.UCANCID] = rev
        }
    }

    return index, scanner.Err()
}

func (r *RevocationIndex) IsRevoked(cid string) bool {
    if r == nil {
        return false
    }
    _, ok := r.items[cid]
    return ok
}

func verifyRevocation(rev Revocation) bool {
    if len(rev.Signature) != ed25519.SignatureSize {
        return false
    }

    pubKey, err := didKeyToPublicKey(rev.RevokedBy)
    if err != nil {
        return false
    }

    payload := revocationPayload{
        UCANCID:   rev.UCANCID,
        RevokedAt: rev.RevokedAt,
        RevokedBy: rev.RevokedBy,
        Reason:    rev.Reason,
    }

    payloadBytes, err := json.Marshal(payload)
    if err != nil {
        return false
    }

    return ed25519.Verify(pubKey, payloadBytes, rev.Signature)
}

func revocationLogPath() (string, error) {
    if home := os.Getenv("LATTICE_HOME"); home != "" {
        return filepath.Join(home, "logs", "revocations.jsonl"), nil
    }
    dir, err := os.UserHomeDir()
    if err != nil {
        return "", err
    }
    return filepath.Join(dir, ".latticefs", "logs", "revocations.jsonl"), nil
}
