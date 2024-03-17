package soupbintcp

import (
	"fmt"
)

const (
  // MaxPayloadLen is the maximum payload length. It's 65534 because the
  // maximum message length is 65535 and 1 byte is needed to encode the packet
  // type.
	MaxPayloadLen = 0xFFFE
)

// Payload is the payload of a packet. It must be at most MaxPayloadLen bytes
// long. Behaviour when too long may vary, but as of right now, it is
// truncated.
type Payload []byte

// NewPayload returns a new payload wrapping the given bytes. Returns an error
// if the payload is too long.
func NewPayload(b []byte) (Payload, error) {
	if len(b) >= MaxPayloadLen {
		return nil, fmt.Errorf("payload too large")
	}
	return Payload(b), nil
}

// Username is a username sent when logging in.
type Username [6]byte

// UsernameFromString creates a new username from the given string. Returns an
// error if the username is too long (longer than 6 bytes). If it is shorter,
// space (' ') bytes are padded on the right to fill the rest.
func UsernameFromString(s string) (Username, error) {
	username := Username{}
	if len(s) > 6 {
		return username, fmt.Errorf("username too long")
	}
	n := copy(username[:], s)
	for ; n < 6; n++ {
		username[n] = ' '
	}
	return username, nil
}

// UsernameFromBytes functions the same as UsernameFromString.
func UsernameFromBytes(b []byte) (Username, error) {
	username := Username{}
	if len(b) > 6 {
		return username, fmt.Errorf("username too long")
	}
	n := copy(username[:], b)
	for ; n < 6; n++ {
		username[n] = ' '
	}
	return username, nil
}

// UsernameFromStringTrunc functions the same as UsernameFromString, but
// truncates the string input if it's too long.
func UsernameFromStringTrunc(s string) Username {
	username := Username{}
	n := copy(username[:], s)
	for ; n < 6; n++ {
		username[n] = ' '
	}
	return username
}

// UsernameFromStringTrunc functions the same as UsernameFromBytesTrunc.
func UsernameFromBytesTrunc(b []byte) Username {
	username := Username{}
	n := copy(username[:], b)
	for ; n < 6; n++ {
		username[n] = ' '
	}
	return username
}

// Password is a password sent when logging in.
type Password [10]byte

// PasswordFromString creates a new password from the given string. Returns an
// error if the password is too long (longer than 10 bytes). If it is shorter,
// space (' ') bytes are padded on the right to fill the rest.
func PasswordFromString(s string) (Password, error) {
	password := Password{}
	if len(s) > 10 {
		return password, fmt.Errorf("password too long")
	}
	n := copy(password[:], s)
	for ; n < 10; n++ {
		password[n] = ' '
	}
	return password, nil
}

// PasswordFromBytes functions the same as PasswordFromString.
func PasswordFromBytes(b []byte) (Password, error) {
	password := Password{}
	if len(b) > 10 {
		return password, fmt.Errorf("password too long")
	}
	n := copy(password[:], b)
	for ; n < 10; n++ {
		password[n] = ' '
	}
	return password, nil
}

// PasswordFromStringTrunc functions the same as PasswordFromString, but
// truncates the string input if it's too long.
func PasswordFromStringTrunc(s string) Password {
	password := Password{}
	n := copy(password[:], s)
	for ; n < 10; n++ {
		password[n] = ' '
	}
	return password
}

// PasswordFromBytesTrunc functions the same as PasswordFromStringTrunc.
func PasswordFromBytesTrunc(b []byte) Password {
	password := Password{}
	n := copy(password[:], b)
	for ; n < 10; n++ {
		password[n] = ' '
	}
	return password
}

// Session represents the session on a server.
type Session [10]byte

// SessionFromString creates a new session from the given bytes. Returns an
// error if the session is too long (longer than 10 bytes). If it is shorter,
// space (' ') bytes are padded on the left to fill the rest.
func SessionFromBytes(b []byte) (Session, error) {
	session, l := Session{}, len(b)
	if l > 10 {
		return session, fmt.Errorf("session too long")
	}
  start := 0
  if l < 10 {
    start = 10 - l
  }
	copy(session[start:], b)
  for i := 0; i < start; i++ {
		session[i] = ' '
	}
	return session, nil
}

// SessionFromStringTrunc functions the same as SessionFromBytes, but truncates
// the bytes input if it's too long.
func SessionFromBytesTrunc(b []byte) Session {
	session, l := Session{}, len(b)
  start := 0
  if l < 10 {
    start = 10 - l
  }
	copy(session[start:], b)
  for i := 0; i < start; i++ {
		session[i] = ' '
	}
	return session
}

// SequenceNumber for sequenced data messages.
type SequenceNumber [20]byte

// SequenceNumberFromBytes creates a new session from the given bytes. Returns
// an error if the sequence number is too long (longer than 20 bytes). If it is
// shorter, space (' ') bytes are padded on the left to fill the rest.
func SequenceNumberFromBytes(b []byte) (SequenceNumber, error) {
	seqNum, l := SequenceNumber{}, len(b)
	if l > 20 {
		return seqNum, fmt.Errorf("session too long")
	}
  start := 0
  if l < 20 {
    start = 20 - l
  }
	copy(seqNum[start:], b)
  for i := 0; i < start; i++ {
		seqNum[i] = ' '
	}
	return seqNum, nil
}

// SequenceNumberFromBytesTrunc functions the same as SequenceNumberFromBytes,
// but truncates the bytes input if it's too long.
func SequenceNumberFromBytesTrunc(b []byte) SequenceNumber {
	seqNum, l := SequenceNumber{}, len(b)
  start := 0
  if l < 20 {
    start = 20 - l
  }
	copy(seqNum[start:], b)
  for i := 0; i < start; i++ {
		seqNum[i] = ' '
	}
  return seqNum
}
