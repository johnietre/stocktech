package soupbintcp_test

import (
	"bytes"
	"fmt"
	"math/rand"
	"strings"
	"testing"
	"time"

	. "github.com/johnietre/stocktech/soupbintcp/v4"
	utils "github.com/johnietre/utils/go"
)

func TestPayload(t *testing.T) {
	var load Payload
	var err error

	load, err = NewPayload(nil)
	if err != nil {
		t.Error("error creating payload: ", err)
	}
	load, err = NewPayload([]byte{})
	if err != nil {
		t.Error("error creating payload: ", err)
	}
	load, err = PayloadFromString("")
	if err != nil {
		t.Error("error creating payload: ", err)
	}
	load, err = NewPayload(make([]byte, MaxPayloadLen))
	if err != nil {
		t.Error("error creating payload: ", err)
	}
	load, err = NewPayload(make([]byte, MaxPayloadLen+1))
	if err == nil {
		t.Error("expected error creating payload")
	}
	load, err = PayloadFromString(string(make([]byte, MaxPayloadLen+1)))
	if err == nil {
		t.Error("expected error creating payload")
	}

	_ = load
}

func TestUsername(t *testing.T) {
	t.Run("TestEmpty", func(t *testing.T) {
		var username Username
		var err error
		username, err = UsernameFromBytes(nil)
		if err != nil {
			t.Error("error creating username: ", err)
		} else if !username.IsValid() {
			t.Errorf("invalid username returned: %v", username)
		}
		username, err = UsernameFromBytes([]byte{})
		if err != nil {
			t.Error("error creating username: ", err)
		} else if !username.IsValid() {
			t.Errorf("invalid username returned: %v", username)
		}
		username, err = UsernameFromString("")
		if err != nil {
			t.Error("error creating username: ", err)
		} else if !username.IsValid() {
			t.Errorf("invalid username returned: %v", username)
		}
		_ = username
	})

	t.Run("TestOk", func(t *testing.T) {
		var busername, susername, wusername Username
		var usernameBytes []byte
		var err error

		usernameBytes = randAlphanumBytes(UsernameLen / 2)
		busername, err = UsernameFromBytes(usernameBytes)
		if err != nil {
			t.Errorf("error creating username for %s: %v", usernameBytes, err)
		}
		susername, err = UsernameFromString(string(usernameBytes))
		if err != nil {
			t.Errorf("error creating username for %s: %v", usernameBytes, err)
		}
		if !busername.Eq(susername) {
			t.Errorf("usernames not equal: %s != %s", busername, susername)
		} else if !busername.IsValid() {
			t.Errorf("invalid username returned: %v", busername)
		}
		copy(
			wusername[:],
			string(usernameBytes)+
				strings.Repeat(" ", UsernameLen-len(usernameBytes)),
		)
		if !busername.Eq(wusername) {
			t.Errorf("expected %v, got %v", wusername, busername)
		}

		usernameBytes = randAlphanumBytes(UsernameLen)
		busername, err = UsernameFromBytes(usernameBytes)
		if err != nil {
			t.Errorf("error creating username for %s: %v", usernameBytes, err)
		}
		susername, err = UsernameFromString(string(usernameBytes))
		if err != nil {
			t.Errorf("error creating username for %s: %v", usernameBytes, err)
		}
		if !busername.Eq(susername) {
			t.Errorf("usernames not equal: %s != %s", busername, susername)
		} else if !busername.IsValid() {
			t.Errorf("invalid username returned: %v", busername)
		}
		copy(wusername[:], string(usernameBytes))
		if !busername.Eq(wusername) {
			t.Errorf("expected %v, got %v", wusername, busername)
		}

		_, _, _ = busername, susername, wusername
	})

	t.Run("TestFail", func(t *testing.T) {
		var busername, susername Username
		var usernameBytes []byte
		var err error

		usernameBytes = randAlphanumBytes(UsernameLen * 2)
		busername, err = UsernameFromBytes(usernameBytes)
		if err == nil {
			t.Errorf(
				"expected creating username for %s, got %s",
				usernameBytes, busername,
			)
		}
		susername, err = UsernameFromString(string(usernameBytes))
		if err == nil {
			t.Errorf(
				"expected creating username for %s, got %s",
				usernameBytes, susername,
			)
		}

		_, _ = busername, susername
	})

	t.Run("TestTrunc", func(t *testing.T) {
		var busername, susername, wusername Username
		var usernameBytes []byte

		usernameBytes = randAlphanumBytes(UsernameLen / 2)
		busername = UsernameFromBytesTrunc(usernameBytes)
		susername = UsernameFromStringTrunc(string(usernameBytes))
		if !busername.Eq(susername) {
			t.Errorf("usernames not equal: %s != %s", busername, susername)
		} else if !busername.IsValid() {
			t.Errorf("invalid username returned: %v", busername)
		}
		copy(
			wusername[:],
			string(usernameBytes)+
				strings.Repeat(" ", UsernameLen-len(usernameBytes)),
		)
		if !busername.Eq(wusername) {
			t.Errorf("expected %v, got %v", wusername, busername)
		}

		usernameBytes = randAlphanumBytes(UsernameLen)
		busername = UsernameFromBytesTrunc(usernameBytes)
		susername = UsernameFromStringTrunc(string(usernameBytes))
		if !busername.Eq(susername) {
			t.Errorf("usernames not equal: %s != %s", busername, susername)
		} else if !busername.IsValid() {
			t.Errorf("invalid username returned: %v", busername)
		}
		copy(wusername[:], string(usernameBytes))
		if !busername.Eq(wusername) {
			t.Errorf("expected %v, got %v", wusername, busername)
		}

		usernameBytes = randAlphanumBytes(UsernameLen * 2)
		busername = UsernameFromBytesTrunc(usernameBytes)
		susername = UsernameFromStringTrunc(string(usernameBytes))
		if !busername.Eq(susername) {
			t.Errorf("usernames not equal: %s != %s", busername, susername)
		} else if !busername.IsValid() {
			t.Errorf("invalid username returned: %v", busername)
		}
		copy(wusername[:], string(usernameBytes[:UsernameLen]))
		if !busername.Eq(wusername) {
			t.Errorf("expected %v, got %v", wusername, busername)
		}

		_, _, _ = busername, susername, wusername
	})

	t.Run("TestIsValid", func(t *testing.T) {
		var username, busername, susername Username
		var usernameBytes []byte
		var err error

		username = Username([UsernameLen]byte{' ', ' ', ' ', ' ', ' ', 'a'})
		if username.IsValid() {
			t.Error("expected invalid username")
		}
		username = Username([UsernameLen]byte{'a', ' ', ' ', ' ', ' ', 'e'})
		if username.IsValid() {
			t.Error("expected invalid username")
		}
		username = Username([UsernameLen]byte{'a', ' ', ' ', ' ', ' ', 0})
		if username.IsValid() {
			t.Error("expected invalid username")
		}
		username = Username([UsernameLen]byte{'a', 'b', 'c', '1', '2', ' '})
		if !username.IsValid() {
			t.Error("expected valid username")
		}
		username = Username([UsernameLen]byte{' ', ' ', ' ', ' ', ' ', ' '})
		if !username.IsValid() {
			t.Error("expected valid username")
		}

		_, _ = busername, susername
		_, _ = usernameBytes, err
	})
}

func TestPassword(t *testing.T) {
	t.Run("TestEmpty", func(t *testing.T) {
		var password Password
		var err error
		password, err = PasswordFromBytes(nil)
		if err != nil {
			t.Error("error creating password: ", err)
		} else if !password.IsValid() {
			t.Errorf("invalid password returned: %v", password)
		}
		password, err = PasswordFromBytes([]byte{})
		if err != nil {
			t.Error("error creating password: ", err)
		} else if !password.IsValid() {
			t.Errorf("invalid password returned: %v", password)
		}
		password, err = PasswordFromString("")
		if err != nil {
			t.Error("error creating password: ", err)
		} else if !password.IsValid() {
			t.Errorf("invalid password returned: %v", password)
		}
		_ = password
	})

	t.Run("TestOk", func(t *testing.T) {
		var bpassword, spassword, wpassword Password
		var passwordBytes []byte
		var err error

		passwordBytes = randAlphanumBytes(PasswordLen / 2)
		bpassword, err = PasswordFromBytes(passwordBytes)
		if err != nil {
			t.Errorf("error creating password for %s: %v", passwordBytes, err)
		}
		spassword, err = PasswordFromString(string(passwordBytes))
		if err != nil {
			t.Errorf("error creating password for %s: %v", passwordBytes, err)
		}
		if !bpassword.Eq(spassword) {
			t.Errorf("passwords not equal: %s != %s", bpassword, spassword)
		} else if !bpassword.IsValid() {
			t.Errorf("invalid password returned: %v", bpassword)
		}
		copy(
			wpassword[:],
			string(passwordBytes)+
				strings.Repeat(" ", PasswordLen-len(passwordBytes)),
		)
		if !bpassword.Eq(wpassword) {
			t.Errorf("expected %v, got %v", wpassword, bpassword)
		}

		passwordBytes = randAlphanumBytes(PasswordLen)
		bpassword, err = PasswordFromBytes(passwordBytes)
		if err != nil {
			t.Errorf("error creating password for %s: %v", passwordBytes, err)
		}
		spassword, err = PasswordFromString(string(passwordBytes))
		if err != nil {
			t.Errorf("error creating password for %s: %v", passwordBytes, err)
		}
		if !bpassword.Eq(spassword) {
			t.Errorf("passwords not equal: %s != %s", bpassword, spassword)
		} else if !bpassword.IsValid() {
			t.Errorf("invalid password returned: %v", bpassword)
		}
		copy(wpassword[:], string(passwordBytes))
		if !bpassword.Eq(wpassword) {
			t.Errorf("expected %v, got %v", wpassword, bpassword)
		}

		_, _, _ = bpassword, spassword, wpassword
	})

	t.Run("TestFail", func(t *testing.T) {
		var bpassword, spassword Password
		var passwordBytes []byte
		var err error

		passwordBytes = randAlphanumBytes(PasswordLen * 2)
		bpassword, err = PasswordFromBytes(passwordBytes)
		if err == nil {
			t.Errorf(
				"expected creating password for %s, got %s",
				passwordBytes, bpassword,
			)
		}
		spassword, err = PasswordFromString(string(passwordBytes))
		if err == nil {
			t.Errorf(
				"expected creating password for %s, got %s",
				passwordBytes, spassword,
			)
		}

		_, _ = bpassword, spassword
	})

	t.Run("TestTrunc", func(t *testing.T) {
		var bpassword, spassword, wpassword Password
		var passwordBytes []byte

		passwordBytes = randAlphanumBytes(PasswordLen / 2)
		bpassword = PasswordFromBytesTrunc(passwordBytes)
		spassword = PasswordFromStringTrunc(string(passwordBytes))
		if !bpassword.Eq(spassword) {
			t.Errorf("passwords not equal: %s != %s", bpassword, spassword)
		} else if !bpassword.IsValid() {
			t.Errorf("invalid password returned: %v", bpassword)
		}
		copy(
			wpassword[:],
			string(passwordBytes)+
				strings.Repeat(" ", PasswordLen-len(passwordBytes)),
		)
		if !bpassword.Eq(wpassword) {
			t.Errorf("expected %v, got %v", wpassword, bpassword)
		}

		passwordBytes = randAlphanumBytes(PasswordLen)
		bpassword = PasswordFromBytesTrunc(passwordBytes)
		spassword = PasswordFromStringTrunc(string(passwordBytes))
		if !bpassword.Eq(spassword) {
			t.Errorf("passwords not equal: %s != %s", bpassword, spassword)
		} else if !bpassword.IsValid() {
			t.Errorf("invalid password returned: %v", bpassword)
		}
		copy(wpassword[:], string(passwordBytes))
		if !bpassword.Eq(wpassword) {
			t.Errorf("expected %v, got %v", wpassword, bpassword)
		}

		passwordBytes = randAlphanumBytes(PasswordLen * 2)
		bpassword = PasswordFromBytesTrunc(passwordBytes)
		spassword = PasswordFromStringTrunc(string(passwordBytes))
		if !bpassword.Eq(spassword) {
			t.Errorf("passwords not equal: %s != %s", bpassword, spassword)
		} else if !bpassword.IsValid() {
			t.Errorf("invalid password returned: %v", bpassword)
		}
		copy(wpassword[:], string(passwordBytes[:PasswordLen]))
		if !bpassword.Eq(wpassword) {
			t.Errorf("expected %v, got %v", wpassword, bpassword)
		}

		_, _, _ = bpassword, spassword, wpassword
	})

	t.Run("TestIsValid", func(t *testing.T) {
		var password, bpassword, spassword Password
		var passwordBytes []byte
		var err error

		password = Password([PasswordLen]byte{
			' ', ' ', ' ', ' ', ' ',
			' ', ' ', ' ', ' ', 'a',
		})
		if password.IsValid() {
			t.Error("expected invalid password")
		}
		password = Password([PasswordLen]byte{
			'a', ' ', ' ', ' ', ' ',
			' ', ' ', ' ', ' ', 'e',
		})
		if password.IsValid() {
			t.Error("expected invalid password")
		}
		password = Password([PasswordLen]byte{
			'a', ' ', ' ', ' ', ' ',
			' ', ' ', ' ', ' ', 0,
		})
		if password.IsValid() {
			t.Error("expected invalid password")
		}
		password = Password([PasswordLen]byte{
			'a', 'b', 'c', 'd', 'e',
			'1', '2', '3', '4', '5',
		})
		if !password.IsValid() {
			t.Error("expected valid password")
		}
		password = Password([PasswordLen]byte{
			'a', 'b', 'c', '1', '2',
			'3', ' ', ' ', ' ', ' ',
		})
		if !password.IsValid() {
			t.Error("expected valid password")
		}
		password = Password([PasswordLen]byte{
			' ', ' ', ' ', ' ', ' ',
			' ', ' ', ' ', ' ', ' ',
		})
		if !password.IsValid() {
			t.Error("expected valid password")
		}

		_, _ = bpassword, spassword
		_, _ = passwordBytes, err
	})
}

func TestSessionId(t *testing.T) {
	t.Run("TestEmpty", func(t *testing.T) {
		var sessionId SessionId
		var err error
		sessionId, err = SessionIdFromBytes(nil)
		if err != nil {
			t.Error("error creating sessionId: ", err)
		} else if !sessionId.IsValid() {
			t.Errorf("invalid sessionId returned: %v", sessionId)
		}
		sessionId, err = SessionIdFromBytes([]byte{})
		if err != nil {
			t.Error("error creating sessionId: ", err)
		} else if !sessionId.IsValid() {
			t.Errorf("invalid sessionId returned: %v", sessionId)
		}
		sessionId, err = SessionIdFromString("")
		if err != nil {
			t.Error("error creating sessionId: ", err)
		} else if !sessionId.IsValid() {
			t.Errorf("invalid sessionId returned: %v", sessionId)
		}
		_ = sessionId
	})

	t.Run("TestOk", func(t *testing.T) {
		var bsessionId, ssessionId, wsessionId SessionId
		var sessionIdBytes []byte
		var err error

		sessionIdBytes = randAlphanumBytes(SessionIdLen / 2)
		bsessionId, err = SessionIdFromBytes(sessionIdBytes)
		if err != nil {
			t.Errorf("error creating sessionId for %s: %v", sessionIdBytes, err)
		}
		ssessionId, err = SessionIdFromString(string(sessionIdBytes))
		if err != nil {
			t.Errorf("error creating sessionId for %s: %v", sessionIdBytes, err)
		}
		if !bsessionId.Eq(ssessionId) {
			t.Errorf("sessionIds not equal: %s != %s", bsessionId, ssessionId)
		} else if !bsessionId.IsValid() {
			t.Errorf("invalid sessionId returned: %v", bsessionId)
		}
		copy(
			wsessionId[:],
			strings.Repeat(" ", SessionIdLen-len(sessionIdBytes))+
				string(sessionIdBytes),
		)
		if !bsessionId.Eq(wsessionId) {
			t.Errorf("expected %v, got %v", wsessionId, bsessionId)
		}

		sessionIdBytes = randAlphanumBytes(SessionIdLen)
		bsessionId, err = SessionIdFromBytes(sessionIdBytes)
		if err != nil {
			t.Errorf("error creating sessionId for %s: %v", sessionIdBytes, err)
		}
		ssessionId, err = SessionIdFromString(string(sessionIdBytes))
		if err != nil {
			t.Errorf("error creating sessionId for %s: %v", sessionIdBytes, err)
		}
		if !bsessionId.Eq(ssessionId) {
			t.Errorf("sessionIds not equal: %s != %s", bsessionId, ssessionId)
		} else if !bsessionId.IsValid() {
			t.Errorf("invalid sessionId returned: %v", bsessionId)
		}
		copy(wsessionId[:], string(sessionIdBytes))
		if !bsessionId.Eq(wsessionId) {
			t.Errorf("expected %v, got %v", wsessionId, bsessionId)
		}

		_, _, _ = bsessionId, ssessionId, wsessionId
	})

	t.Run("TestFail", func(t *testing.T) {
		var bsessionId, ssessionId SessionId
		var sessionIdBytes []byte
		var err error

		sessionIdBytes = randAlphanumBytes(SessionIdLen * 2)
		bsessionId, err = SessionIdFromBytes(sessionIdBytes)
		if err == nil {
			t.Errorf(
				"expected creating sessionId for %s, got %s",
				sessionIdBytes, bsessionId,
			)
		}
		ssessionId, err = SessionIdFromString(string(sessionIdBytes))
		if err == nil {
			t.Errorf(
				"expected creating sessionId for %s, got %s",
				sessionIdBytes, ssessionId,
			)
		}

		_, _ = bsessionId, ssessionId
	})

	t.Run("TestTrunc", func(t *testing.T) {
		var bsessionId, ssessionId, wsessionId SessionId
		var sessionIdBytes []byte

		sessionIdBytes = randAlphanumBytes(SessionIdLen / 2)
		bsessionId = SessionIdFromBytesTrunc(sessionIdBytes)
		ssessionId = SessionIdFromStringTrunc(string(sessionIdBytes))
		if !bsessionId.Eq(ssessionId) {
			t.Errorf("sessionIds not equal: %s != %s", bsessionId, ssessionId)
		} else if !bsessionId.IsValid() {
			t.Errorf("invalid sessionId returned: %v", bsessionId)
		}
		copy(
			wsessionId[:],
			strings.Repeat(" ", SessionIdLen-len(sessionIdBytes))+
				string(sessionIdBytes),
		)
		if !bsessionId.Eq(wsessionId) {
			t.Errorf("expected %v, got %v", wsessionId, bsessionId)
		}

		sessionIdBytes = randAlphanumBytes(SessionIdLen)
		bsessionId = SessionIdFromBytesTrunc(sessionIdBytes)
		ssessionId = SessionIdFromStringTrunc(string(sessionIdBytes))
		if !bsessionId.Eq(ssessionId) {
			t.Errorf("sessionIds not equal: %s != %s", bsessionId, ssessionId)
		} else if !bsessionId.IsValid() {
			t.Errorf("invalid sessionId returned: %v", bsessionId)
		}
		copy(wsessionId[:], string(sessionIdBytes))
		if !bsessionId.Eq(wsessionId) {
			t.Errorf("expected %v, got %v", wsessionId, bsessionId)
		}

		sessionIdBytes = randAlphanumBytes(SessionIdLen * 2)
		bsessionId = SessionIdFromBytesTrunc(sessionIdBytes)
		ssessionId = SessionIdFromStringTrunc(string(sessionIdBytes))
		if !bsessionId.Eq(ssessionId) {
			t.Errorf("sessionIds not equal: %s != %s", bsessionId, ssessionId)
		} else if !bsessionId.IsValid() {
			t.Errorf("invalid sessionId returned: %v", bsessionId)
		}
		copy(wsessionId[:], string(sessionIdBytes[:SessionIdLen]))
		if !bsessionId.Eq(wsessionId) {
			t.Errorf("expected %v, got %v", wsessionId, bsessionId)
		}

		_, _, _ = bsessionId, ssessionId, wsessionId
	})

	t.Run("TestIsValid", func(t *testing.T) {
		var sessionId, bsessionId, ssessionId SessionId
		var sessionIdBytes []byte
		var err error

		sessionId = SessionId([SessionIdLen]byte{
			'a', ' ', ' ', ' ', ' ',
			' ', ' ', ' ', ' ', ' ',
		})
		if sessionId.IsValid() {
			t.Error("expected invalid sessionId")
		}
		sessionId = SessionId([SessionIdLen]byte{
			'a', ' ', ' ', ' ', ' ',
			' ', ' ', ' ', ' ', 'e',
		})
		if sessionId.IsValid() {
			t.Error("expected invalid sessionId")
		}
		sessionId = SessionId([SessionIdLen]byte{
			'a', ' ', ' ', ' ', ' ',
			' ', ' ', ' ', ' ', 0,
		})
		if sessionId.IsValid() {
			t.Error("expected invalid sessionId")
		}
		sessionId = SessionId([SessionIdLen]byte{
			'a', 'b', 'c', 'd', 'e',
			'1', '2', '3', '4', '5',
		})
		if !sessionId.IsValid() {
			t.Error("expected valid sessionId")
		}
		sessionId = SessionId([SessionIdLen]byte{
			' ', ' ', ' ', ' ', 'a',
			'b', 'c', '1', '2', '3',
		})
		if !sessionId.IsValid() {
			t.Error("expected valid sessionId")
		}
		sessionId = SessionId([SessionIdLen]byte{
			' ', ' ', ' ', ' ', ' ',
			' ', ' ', ' ', ' ', ' ',
		})
		if !sessionId.IsValid() {
			t.Error("expected valid sessionId")
		}

		_, _ = bsessionId, ssessionId
		_, _ = sessionIdBytes, err
	})
}

func TestSequenceNumber(t *testing.T) {
	t.Run("TestEmpty", func(t *testing.T) {
		var seqNum SequenceNumber
		var err error
		seqNum, err = SequenceNumberFromBytes(nil)
		if err != nil {
			t.Error("error creating seqNum: ", err)
		} else if !seqNum.IsValid() {
			t.Errorf("invalid seqNum returned: %v", seqNum)
		}
		seqNum, err = SequenceNumberFromBytes([]byte{})
		if err != nil {
			t.Error("error creating seqNum: ", err)
		} else if !seqNum.IsValid() {
			t.Errorf("invalid seqNum returned: %v", seqNum)
		}
		seqNum, err = SequenceNumberFromString("")
		if err != nil {
			t.Error("error creating seqNum: ", err)
		} else if !seqNum.IsValid() {
			t.Errorf("invalid seqNum returned: %v", seqNum)
		}
		_ = seqNum
	})

	t.Run("TestOk", func(t *testing.T) {
		var bseqNum, sseqNum, wseqNum SequenceNumber
		var seqNumBytes []byte
		var err error

		seqNumBytes = randNumBytes(SequenceNumberLen / 2)
		bseqNum, err = SequenceNumberFromBytes(seqNumBytes)
		if err != nil {
			t.Errorf("error creating seqNum for %s: %v", seqNumBytes, err)
		}
		sseqNum, err = SequenceNumberFromString(string(seqNumBytes))
		if err != nil {
			t.Errorf("error creating seqNum for %s: %v", seqNumBytes, err)
		}
		if !bseqNum.Eq(sseqNum) {
			t.Errorf("seqNums not equal: %s != %s", bseqNum, sseqNum)
		} else if !bseqNum.IsValid() {
			t.Errorf("invalid seqNum returned: %v", bseqNum)
		}
		copy(
			wseqNum[:],
			strings.Repeat(" ", SequenceNumberLen-len(seqNumBytes))+
				string(seqNumBytes),
		)
		if !bseqNum.Eq(wseqNum) {
			t.Errorf("expected %v, got %v", wseqNum, bseqNum)
		}

		seqNumBytes = randNumBytes(SequenceNumberLen)
		bseqNum, err = SequenceNumberFromBytes(seqNumBytes)
		if err != nil {
			t.Errorf("error creating seqNum for %s: %v", seqNumBytes, err)
		}
		sseqNum, err = SequenceNumberFromString(string(seqNumBytes))
		if err != nil {
			t.Errorf("error creating seqNum for %s: %v", seqNumBytes, err)
		}
		if !bseqNum.Eq(sseqNum) {
			t.Errorf("seqNums not equal: %s != %s", bseqNum, sseqNum)
		} else if !bseqNum.IsValid() {
			t.Errorf("invalid seqNum returned: %v", bseqNum)
		}
		copy(wseqNum[:], string(seqNumBytes))
		if !bseqNum.Eq(wseqNum) {
			t.Errorf("expected %v, got %v", wseqNum, bseqNum)
		}

		_, _, _ = bseqNum, sseqNum, wseqNum
	})

	t.Run("TestFail", func(t *testing.T) {
		var bseqNum, sseqNum SequenceNumber
		var seqNumBytes []byte
		var err error

		seqNumBytes = randNumBytes(SequenceNumberLen * 2)
		bseqNum, err = SequenceNumberFromBytes(seqNumBytes)
		if err == nil {
			t.Errorf(
				"expected creating seqNum for %s, got %s",
				seqNumBytes, bseqNum,
			)
		}
		sseqNum, err = SequenceNumberFromString(string(seqNumBytes))
		if err == nil {
			t.Errorf(
				"expected creating seqNum for %s, got %s",
				seqNumBytes, sseqNum,
			)
		}

		_, _ = bseqNum, sseqNum
	})

	t.Run("TestTrunc", func(t *testing.T) {
		var bseqNum, sseqNum, wseqNum SequenceNumber
		var seqNumBytes []byte

		seqNumBytes = randNumBytes(SequenceNumberLen / 2)
		bseqNum = SequenceNumberFromBytesTrunc(seqNumBytes)
		sseqNum = SequenceNumberFromStringTrunc(string(seqNumBytes))
		if !bseqNum.Eq(sseqNum) {
			t.Errorf("seqNums not equal: %s != %s", bseqNum, sseqNum)
		} else if !bseqNum.IsValid() {
			t.Errorf("invalid seqNum returned: %v", bseqNum)
		}
		copy(
			wseqNum[:],
			strings.Repeat(" ", SequenceNumberLen-len(seqNumBytes))+
				string(seqNumBytes),
		)
		if !bseqNum.Eq(wseqNum) {
			t.Errorf("expected %v, got %v", wseqNum, bseqNum)
		}

		seqNumBytes = randNumBytes(SequenceNumberLen)
		bseqNum = SequenceNumberFromBytesTrunc(seqNumBytes)
		sseqNum = SequenceNumberFromStringTrunc(string(seqNumBytes))
		if !bseqNum.Eq(sseqNum) {
			t.Errorf("seqNums not equal: %s != %s", bseqNum, sseqNum)
		} else if !bseqNum.IsValid() {
			t.Errorf("invalid seqNum returned: %v", bseqNum)
		}
		copy(wseqNum[:], string(seqNumBytes))
		if !bseqNum.Eq(wseqNum) {
			t.Errorf("expected %v, got %v", wseqNum, bseqNum)
		}

		seqNumBytes = randNumBytes(SequenceNumberLen * 2)
		bseqNum = SequenceNumberFromBytesTrunc(seqNumBytes)
		sseqNum = SequenceNumberFromStringTrunc(string(seqNumBytes))
		if !bseqNum.Eq(sseqNum) {
			t.Errorf("seqNums not equal: %s != %s", bseqNum, sseqNum)
		} else if !bseqNum.IsValid() {
			t.Errorf("invalid seqNum returned: %v", bseqNum)
		}
		copy(wseqNum[:], string(seqNumBytes[:SequenceNumberLen]))
		if !bseqNum.Eq(wseqNum) {
			t.Errorf("expected %v, got %v", wseqNum, bseqNum)
		}

		_, _, _ = bseqNum, sseqNum, wseqNum
	})

	t.Run("TestIsValid", func(t *testing.T) {
		var seqNum, bseqNum, sseqNum SequenceNumber
		var seqNumBytes []byte
		var err error

		seqNum = SequenceNumber([SequenceNumberLen]byte{
			'a', ' ', ' ', ' ', ' ',
			' ', ' ', ' ', ' ', ' ',
			' ', ' ', ' ', ' ', ' ',
			' ', ' ', ' ', ' ', ' ',
		})
		if seqNum.IsValid() {
			t.Error("expected invalid seqNum")
		}
		seqNum = SequenceNumber([SequenceNumberLen]byte{
			'2', ' ', ' ', ' ', ' ',
			' ', ' ', ' ', ' ', ' ',
			' ', ' ', ' ', ' ', ' ',
			' ', ' ', ' ', ' ', '1',
		})
		if seqNum.IsValid() {
			t.Error("expected invalid seqNum")
		}
		seqNum = SequenceNumber([SequenceNumberLen]byte{
			' ', ' ', ' ', ' ', ' ',
			' ', ' ', ' ', ' ', ' ',
			' ', ' ', ' ', ' ', ' ',
			' ', ' ', ' ', ' ', 0,
		})
		if seqNum.IsValid() {
			t.Error("expected invalid seqNum")
		}
		seqNum = SequenceNumber([SequenceNumberLen]byte{
			'1', '2', '3', '4', '5',
			'1', '2', '3', '4', '5',
			'1', '2', '3', '4', '5',
			'1', '2', '3', '4', '5',
		})
		if !seqNum.IsValid() {
			t.Error("expected valid seqNum")
		}
		seqNum = SequenceNumber([SequenceNumberLen]byte{
			' ', ' ', ' ', ' ', ' ',
			' ', ' ', ' ', ' ', ' ',
			' ', ' ', ' ', ' ', '1',
			'2', '3', '4', '5', '6',
		})
		if !seqNum.IsValid() {
			t.Error("expected valid seqNum")
		}
		seqNum = SequenceNumber([SequenceNumberLen]byte{
			' ', ' ', ' ', ' ', ' ',
			' ', ' ', ' ', ' ', ' ',
			' ', ' ', ' ', ' ', ' ',
			' ', ' ', ' ', ' ', '0',
		})
		if !seqNum.IsValid() {
			t.Error("expected valid seqNum")
		}

		_, _ = bseqNum, sseqNum
		_, _ = seqNumBytes, err
	})

	t.Run("TestToFro", func(t *testing.T) {
		var seqNum SequenceNumber
		var num uint64
		var err error
		var ok bool

		num = 123456789
		seqNum = SequenceNumberFromUint64(num)
		if !seqNum.IsValid() {
			t.Errorf("invalid seqNum returned: %v", seqNum)
		} else if n := seqNum.ToUint64(); n != num {
			t.Errorf("expected %d, got %d", num, n)
		} else if n, ok := seqNum.ToUint64Safe(); !ok {
			t.Errorf("safe failed")
		} else if n != num {
			t.Errorf("expected %d, got %d", num, n)
		}

		num = 0
		seqNum = SequenceNumberFromUint64(num)
		if !seqNum.IsValid() {
			t.Errorf("invalid seqNum returned: %v", seqNum)
		} else if n := seqNum.ToUint64(); n != num {
			t.Errorf("expected %d, got %d", num, n)
		} else if n, ok := seqNum.ToUint64Safe(); !ok {
			t.Errorf("safe failed")
		} else if n != num {
			t.Errorf("expected %d, got %d", num, n)
		}

		num = 0
		seqNum = SequenceNumberZero()
		if n := seqNum.ToUint64(); n != num {
			t.Errorf("expected %d, got %d", num, n)
		} else if n, ok := seqNum.ToUint64Safe(); !ok {
			t.Errorf("safe failed")
		} else if n != num {
			t.Errorf("expected %d, got %d", num, n)
		}

		num = 123456789
		seqNum = SequenceNumberFromStringTrunc("    123456789")
		if n := seqNum.ToUint64(); n != num {
			t.Errorf("expected %d, got %d", num, n)
		} else if n, ok := seqNum.ToUint64Safe(); !ok {
			t.Errorf("safe failed")
		} else if n != num {
			t.Errorf("expected %d, got %d", num, n)
		}

		num = 123456789
		seqNum = SequenceNumberFromStringTrunc("0000123456789")
		if n := seqNum.ToUint64(); n != num {
			t.Errorf("expected %d, got %d", num, n)
		} else if n, ok := seqNum.ToUint64Safe(); !ok {
			t.Errorf("safe failed")
		} else if n != num {
			t.Errorf("expected %d, got %d", num, n)
		}

		num = 0
		seqNum = SequenceNumberFromStringTrunc("0000000000000")
		if n := seqNum.ToUint64(); n != num {
			t.Errorf("expected %d, got %d", num, n)
		} else if n, ok := seqNum.ToUint64Safe(); !ok {
			t.Errorf("safe failed")
		} else if n != num {
			t.Errorf("expected %d, got %d", num, n)
		}

		seqNum = SequenceNumberFromStringTrunc("00000  000000")
		if n, ok := seqNum.ToUint64Safe(); ok {
			t.Errorf("expected failed safe, got: %d", n)
		}

		seqNum = SequenceNumberFromStringTrunc("1            ")
		if n, ok := seqNum.ToUint64Safe(); ok {
			t.Errorf("expected failed safe, got: %d", n)
		}

		seqNum = SequenceNumberFromStringTrunc("123456789a")
		if n, ok := seqNum.ToUint64Safe(); ok {
			t.Errorf("expected failed safe, got: %d", n)
		}

		_, _, _, _ = seqNum, num, err, ok
	})
}

func TestPacket(t *testing.T) {
	buf := bytes.NewBuffer(nil)

	// NRW = New, Read, Write
	t.Run("TestNRWOk", func(t *testing.T) {
		var packet Packet

		// Debug
		packet = DebugPacket(utils.First(PayloadFromString("")))
		buf.Write(packet.Bytes())
		if l := 3 + len(packet.Payload()); l != buf.Len() {
			t.Errorf("expected %d bytes written, got %d", l, buf.Len())
		} else if err := doParseRead(buf); err != nil {
			t.Error("error reading/parsing packet: ", err)
		}
		packet = DebugPacket(utils.First(NewPayload(randAlphanumBytes(10000))))
		buf.Write(packet.Bytes())
		if l := 3 + len(packet.Payload()); l != buf.Len() {
			t.Errorf("expected %d bytes written, got %d", l, buf.Len())
		} else if err := doParseRead(buf); err != nil {
			t.Error("error reading/parsing packet: ", err)
		}

		// LoginAccepted
		packet = LoginAcceptedPacket(SessionIdBlank(), SequenceNumberZero())
		buf.Write(packet.Bytes())
		if l := 3 + len(packet.Payload()); l != buf.Len() {
			t.Errorf("expected %d bytes written, got %d", l, buf.Len())
		} else if err := doParseRead(buf); err != nil {
			t.Error("error reading/parsing packet: ", err)
		}

		// LoginReject
		packet = LoginRejectPacket(LoginRejectNotAuthorized)
		buf.Write(packet.Bytes())
		if l := 3 + len(packet.Payload()); l != buf.Len() {
			t.Errorf("expected %d bytes written, got %d", l, buf.Len())
		} else if err := doParseRead(buf); err != nil {
			t.Error("error reading/parsing packet: ", err)
		}

		// Sequenced
		packet = SequencedDataPacket(utils.First(PayloadFromString("")))
		buf.Write(packet.Bytes())
		if l := 3 + len(packet.Payload()); l != buf.Len() {
			t.Errorf("expected %d bytes written, got %d", l, buf.Len())
		} else if err := doParseRead(buf); err != nil {
			t.Error("error reading/parsing packet: ", err)
		}
		packet = SequencedDataPacket(utils.First(
			NewPayload(randAlphanumBytes(10000)),
		))
		buf.Write(packet.Bytes())
		if l := 3 + len(packet.Payload()); l != buf.Len() {
			t.Errorf("expected %d bytes written, got %d", l, buf.Len())
		} else if err := doParseRead(buf); err != nil {
			t.Error("error reading/parsing packet: ", err)
		}

		// Unsequenced
		packet = UnsequencedDataPacket(utils.First(PayloadFromString("")))
		buf.Write(packet.Bytes())
		if l := 3 + len(packet.Payload()); l != buf.Len() {
			t.Errorf("expected %d bytes written, got %d", l, buf.Len())
		} else if err := doParseRead(buf); err != nil {
			t.Error("error reading/parsing packet: ", err)
		}
		packet = UnsequencedDataPacket(utils.First(
			NewPayload(randAlphanumBytes(10000)),
		))
		buf.Write(packet.Bytes())
		if l := 3 + len(packet.Payload()); l != buf.Len() {
			t.Errorf("expected %d bytes written, got %d", l, buf.Len())
		} else if err := doParseRead(buf); err != nil {
			t.Error("error reading/parsing packet: ", err)
		}

		// ServerHeartbeat
		packet = ServerHeartbeatPacket()
		buf.Write(packet.Bytes())
		if l := 3 + len(packet.Payload()); l != buf.Len() {
			t.Errorf("expected %d bytes written, got %d", l, buf.Len())
		} else if err := doParseRead(buf); err != nil {
			t.Error("error reading/parsing packet: ", err)
		}

		// ClientHeartbeat
		packet = ClientHeartbeatPacket()
		buf.Write(packet.Bytes())
		if l := 3 + len(packet.Payload()); l != buf.Len() {
			t.Errorf("expected %d bytes written, got %d", l, buf.Len())
		} else if err := doParseRead(buf); err != nil {
			t.Error("error reading/parsing packet: ", err)
		}

		// EndOfSession
		packet = EndOfSessionPacket()
		buf.Write(packet.Bytes())
		if l := 3 + len(packet.Payload()); l != buf.Len() {
			t.Errorf("expected %d bytes written, got %d", l, buf.Len())
		} else if err := doParseRead(buf); err != nil {
			t.Error("error reading/parsing packet: ", err)
		}

		// LogoutRequest
		packet = LogoutRequestPacket()
		buf.Write(packet.Bytes())
		if l := 3 + len(packet.Payload()); l != buf.Len() {
			t.Errorf("expected %d bytes written, got %d", l, buf.Len())
		} else if err := doParseRead(buf); err != nil {
			t.Error("error reading/parsing packet: ", err)
		}

		// LoginRequest
		packet = LoginRequestPacket(
			UsernameFromBytesTrunc(randAlphanumBytes(UsernameLen)),
			PasswordFromBytesTrunc(randAlphanumBytes(PasswordLen)),
			SessionIdFromBytesTrunc(randAlphanumBytes(SessionIdLen)),
			SequenceNumberFromBytesTrunc(randNumBytes(SequenceNumberLen)),
		)
		buf.Write(packet.Bytes())
		if l := 3 + len(packet.Payload()); l != buf.Len() {
			t.Errorf("expected %d bytes written, got %d", l, buf.Len())
		} else if err := doParseRead(buf); err != nil {
			t.Error("error reading/parsing packet: ", err)
		}
	})

	t.Run("TestTry", func(t *testing.T) {
		packetTypes := []PacketType{
			PacketTypeDebug,
			PacketTypeLoginAccepted,
			PacketTypeLoginReject,
			PacketTypeSequencedData,
			PacketTypeUnsequencedData,
			PacketTypeServerHeartbeat,
			PacketTypeEndOfSession,
			PacketTypeLoginRequest,
			PacketTypeClientHeartbeat,
			PacketTypeLogoutRequest,
		}
		runTest := func(packet Packet) {
			for _, pt := range packetTypes {
				buf := bytes.NewBuffer(nil)
				buf.Write(packet.Bytes())
				checkErr := func(err error) error {
					if utils.ErrAs[*UnexpectedPacketTypeError](err) {
						return nil
					}
					return err
				}
				if packet.PacketType() == pt {
					checkErr = func(err error) error {
						return err
					}
				}
				buf.Write(packet.Bytes())
				err := doTryParseRead(
					buf,
					packet.PacketType(),
					checkErr,
				)
				if err != nil {
					t.Error(err)
				}
			}
		}

		runTest(DebugPacket(emptyPayload()))
		runTest(LoginAcceptedPacket(SessionIdBlank(), SequenceNumberZero()))
		runTest(LoginRejectPacket(LoginRejectNotAuthorized))
		runTest(SequencedDataPacket(emptyPayload()))
		runTest(UnsequencedDataPacket(emptyPayload()))
		runTest(ServerHeartbeatPacket())
		runTest(EndOfSessionPacket())
		runTest(LoginRequestPacket(
			UsernameFromBytesTrunc(randAlphanumBytes(UsernameLen)),
			PasswordFromBytesTrunc(randAlphanumBytes(PasswordLen)),
			SessionIdFromBytesTrunc(randAlphanumBytes(SessionIdLen)),
			SequenceNumberFromBytesTrunc(randNumBytes(SequenceNumberLen)),
		))
		runTest(ClientHeartbeatPacket())
		runTest(LogoutRequestPacket())
	})

	t.Run("TestFields", func(t *testing.T) {
		payload := utils.First(NewPayload(randAlphanumBytes(MaxPayloadLen / 2)))
		username := UsernameFromBytesTrunc(randAlphanumBytes(UsernameLen))
		password := PasswordFromBytesTrunc(randAlphanumBytes(PasswordLen))
		sessionId := SessionIdFromBytesTrunc(randAlphanumBytes(SessionIdLen))
		seqNum := SequenceNumberFromBytesTrunc(randNumBytes(SequenceNumberLen))
		reject := LoginRejectSessionNotAvail

		runTest := func(packet Packet) {
			var u Username
			var p Password
			var si SessionId
			var sn SequenceNumber
			var lr LoginReject
			var ok, is bool

			pt := packet.PacketType()
			is = pt == PacketTypeLoginRequest

			u, ok = packet.Username()
			if ok != is {
				t.Errorf(
					"expected ok = %v, got %v for %s",
					ok, is, pt,
				)
			} else if ok && !u.Eq(username) {
				t.Errorf("expected username %v, got %v", username, u)
			}
			p, ok = packet.Password()
			if ok != is {
				t.Errorf(
					"expected ok = %v, got %v for %s",
					ok, is, pt,
				)
			} else if ok && !p.Eq(password) {
				t.Errorf("expected password %v, got %v", password, p)
			}
			u, p, ok = packet.Credentials()
			if ok != is {
				t.Errorf(
					"expected ok = %v, got %v for %s",
					ok, is, pt,
				)
			} else if ok && !u.Eq(username) {
				t.Errorf("expected username %v, got %v", username, u)
			} else if ok && !p.Eq(password) {
				t.Errorf("expected password %v, got %v", password, p)
			}

			is = pt == PacketTypeLoginRequest || pt == PacketTypeLoginAccepted
			si, ok = packet.SessionId()
			if ok != is {
				t.Errorf(
					"expected ok = %v, got %v for %s",
					ok, is, pt,
				)
			} else if ok && !si.Eq(sessionId) {
				t.Errorf("expected sessionId %v, got %v", sessionId, si)
			}
			sn, ok = packet.SequenceNumber()
			if ok != is {
				t.Errorf(
					"expected ok = %v, got %v for %s",
					ok, is, pt,
				)
			} else if ok && !sn.Eq(seqNum) {
				t.Errorf("expected seqNum %v, got %v", seqNum, sn)
			}

			is = pt == PacketTypeLoginReject
			lr, ok = packet.RejectReason()
			if ok != is {
				t.Errorf(
					"expected ok = %v, got %v for %s",
					ok, is, pt,
				)
			} else if ok && lr != reject {
				t.Errorf("expected login reject %v, got %v", reject, lr)
			}
		}

		packets := []Packet{
			DebugPacket(payload),
			LoginAcceptedPacket(sessionId, seqNum),
			LoginRejectPacket(reject),
			SequencedDataPacket(payload),
			UnsequencedDataPacket(payload),
			ServerHeartbeatPacket(),
			EndOfSessionPacket(),
			LoginRequestPacket(username, password, sessionId, seqNum),
			ClientHeartbeatPacket(),
			LogoutRequestPacket(),
		}
		for _, packet := range packets {
			runTest(packet)
		}
	})
	// TODO: test errors
}

func doParseRead(buf *bytes.Buffer) error {
	pp, err := ParsePacket(buf.Bytes())
	if err != nil {
		return fmt.Errorf("ParsePacket: %v", err)
	}
	rp, err := ReadPacketFrom(buf)
	if err != nil {
		return fmt.Errorf("ReadPacketFrom: %v", err)
	}
	if !bytes.Equal(pp.Bytes(), rp.Bytes()) {
		err = fmt.Errorf(
			"mismatch bytes (lengths: %d, %d)",
			len(pp.Bytes()), len(rp.Bytes()),
		)
	}
	return err
}

func doTryParseRead(
	buf *bytes.Buffer,
	want PacketType,
	checkErr func(err error) error,
) error {
	pp, err := TryParsePacketAs(buf.Bytes(), want)
	if err = checkErr(err); err != nil {
		return fmt.Errorf("TryParsePacketAs: %v", err)
	}
	rp, err := TryReadPacketFromAs(buf, want)
	if err = checkErr(err); err != nil {
		return fmt.Errorf("TryReadPacketFromAs: %v", err)
	}
	if !bytes.Equal(pp.Bytes(), rp.Bytes()) {
		err = fmt.Errorf(
			"mismatch bytes (lengths: %d, %d)",
			len(pp.Bytes()), len(rp.Bytes()),
		)
	}
	return err
}

func randAlphanumBytes(l int) []byte {
	rng := rand.New(rand.NewSource(time.Now().UnixNano()))
	b := make([]byte, l)
	for i := 0; i < l; i++ {
		n := rng.Intn(10 + 26 + 26)
		if n < 10 {
			b[i] = byte(n) + '0'
		} else if n < 36 {
			b[i] = byte(n-10) + 'A'
		} else {
			b[i] = byte(n-36) + 'a'
		}
	}
	return b
}

func randNumBytes(l int) []byte {
	rng := rand.New(rand.NewSource(time.Now().UnixNano()))
	b := make([]byte, l)
	for i := 0; i < l; i++ {
		n := rng.Intn(10)
		b[i] = byte(n) + '0'
	}
	if l >= SequenceNumberLen {
		// Don't allow overflows
		b[0] = '0'
	}
	return b
}

func emptyPayload() Payload {
	return utils.First(NewPayload(nil))
}
