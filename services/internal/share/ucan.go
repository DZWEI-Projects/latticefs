package share

import (
    "crypto/ed25519"
    "encoding/base64"
    "encoding/json"
    "errors"
    "fmt"
    "strings"
    "time"

    "github.com/mr-tron/base58"
    "github.com/zeebo/blake3"
)

const (
    ucanVersion          = "0.10.0"
    maxProofChainDepth   = 10
    clockSkewSeconds int = 300
)

type UcanHeader struct {
    Alg string `json:"alg"`
    Typ string `json:"typ"`
    Ucv string `json:"ucv"`
}

type Attenuation struct {
    With string `json:"with"`
    Can  string `json:"can"`
}

type UcanPayload struct {
    Iss string        `json:"iss"`
    Aud string        `json:"aud"`
    Sub string        `json:"sub,omitempty"`
    Exp int64         `json:"exp"`
    Nbf *int64        `json:"nbf,omitempty"`
    Nnc string        `json:"nnc,omitempty"`
    Att []Attenuation `json:"att"`
    Prf []string      `json:"prf,omitempty"`
    Fct map[string]any `json:"fct,omitempty"`
}

type Ucan struct {
    Token     string
    Header    UcanHeader
    Payload   UcanPayload
    Signature []byte
}

func ParseUcan(token string) (*Ucan, error) {
    parts := strings.Split(token, ".")
    if len(parts) != 3 {
        return nil, errors.New("invalid UCAN format")
    }

    headerBytes, err := base64.RawURLEncoding.DecodeString(parts[0])
    if err != nil {
        return nil, fmt.Errorf("header decode: %w", err)
    }
    payloadBytes, err := base64.RawURLEncoding.DecodeString(parts[1])
    if err != nil {
        return nil, fmt.Errorf("payload decode: %w", err)
    }
    signature, err := base64.RawURLEncoding.DecodeString(parts[2])
    if err != nil {
        return nil, fmt.Errorf("signature decode: %w", err)
    }

    var header UcanHeader
    if err := json.Unmarshal(headerBytes, &header); err != nil {
        return nil, fmt.Errorf("header parse: %w", err)
    }
    var payload UcanPayload
    if err := json.Unmarshal(payloadBytes, &payload); err != nil {
        return nil, fmt.Errorf("payload parse: %w", err)
    }

    return &Ucan{
        Token:     token,
        Header:    header,
        Payload:   payload,
        Signature: signature,
    }, nil
}

func ValidateUcan(token string, revocations *RevocationIndex) (*Ucan, error) {
    return validateUcan(token, revocations, 0)
}

func validateUcan(token string, revocations *RevocationIndex, depth int) (*Ucan, error) {
    if depth > maxProofChainDepth {
        return nil, errors.New("proof chain too deep")
    }

    ucan, err := ParseUcan(token)
    if err != nil {
        return nil, err
    }

    if ucan.Header.Alg != "EdDSA" || ucan.Header.Typ != "JWT" || ucan.Header.Ucv != ucanVersion {
        return nil, errors.New("unsupported UCAN header")
    }

    signingInput := strings.Join(strings.Split(token, ".")[:2], ".")
    pubKey, err := didKeyToPublicKey(ucan.Payload.Iss)
    if err != nil {
        return nil, err
    }
    if len(ucan.Signature) != ed25519.SignatureSize {
        return nil, errors.New("invalid signature length")
    }
    if !ed25519.Verify(pubKey, []byte(signingInput), ucan.Signature) {
        return nil, errors.New("invalid signature")
    }

    now := time.Now().Unix()
    if now >= ucan.Payload.Exp+int64(clockSkewSeconds) {
        return nil, errors.New("capability expired")
    }
    if ucan.Payload.Nbf != nil && now+int64(clockSkewSeconds) < *ucan.Payload.Nbf {
        return nil, errors.New("capability not yet valid")
    }

    if revocations != nil {
        cid := ucanCID(token)
        if revocations.IsRevoked(cid) {
            return nil, errors.New("capability revoked")
        }
    }

    for _, proofToken := range ucan.Payload.Prf {
        proof, err := validateUcan(proofToken, revocations, depth+1)
        if err != nil {
            return nil, fmt.Errorf("invalid proof: %w", err)
        }
        if ucan.Payload.Iss != proof.Payload.Aud {
            return nil, errors.New("issuer is not audience of proof")
        }

        for _, att := range ucan.Payload.Att {
            if !proofAllows(proof.Payload.Att, att) {
                return nil, errors.New("invalid attenuation")
            }
        }
    }

    return ucan, nil
}

func didKeyToPublicKey(did string) (ed25519.PublicKey, error) {
    if !strings.HasPrefix(did, "did:key:z") {
        return nil, errors.New("unsupported DID")
    }
    encoded := strings.TrimPrefix(did, "did:key:z")
    decoded, err := base58.Decode(encoded)
    if err != nil {
        return nil, fmt.Errorf("did decode: %w", err)
    }
    if len(decoded) < 34 {
        return nil, errors.New("invalid did:key length")
    }
    // Remove multicodec prefix (0xed01 for Ed25519)
    pub := decoded[2:34]
    if len(pub) != ed25519.PublicKeySize {
        return nil, errors.New("invalid public key length")
    }
    return ed25519.PublicKey(pub), nil
}

func permissionLevel(p string) int {
    switch strings.ToLower(p) {
    case "read":
        return 1
    case "comment":
        return 2
    case "write":
        return 3
    case "share":
        return 4
    case "admin":
        return 5
    default:
        return 0
    }
}

func IsValidPermission(p string) bool {
    return permissionLevel(p) > 0
}

func proofAllows(proof []Attenuation, requested Attenuation) bool {
    for _, att := range proof {
        if !resourceMatches(att.With, requested.With) {
            continue
        }
        if permissionLevel(att.Can) >= permissionLevel(requested.Can) {
            return true
        }
    }
    return false
}

func resourceMatches(scope string, target string) bool {
    if scope == target {
        return true
    }
    if strings.HasSuffix(scope, "*") {
        prefix := strings.TrimSuffix(scope, "*")
        return strings.HasPrefix(target, prefix)
    }
    return false
}

func ucanCID(token string) string {
    sum := blake3.Sum256([]byte(token))
    return fmt.Sprintf("%x", sum[:])
}

func HasPermission(ucan *Ucan, resource string, permission string) bool {
    for _, att := range ucan.Payload.Att {
        if resourceMatches(att.With, resource) && permissionLevel(att.Can) >= permissionLevel(permission) {
            return true
        }
    }
    return false
}
