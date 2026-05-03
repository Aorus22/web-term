package ssh

import (
	"bytes"
	"testing"
)

func TestRingBuffer(t *testing.T) {
	rb := NewRingBuffer(10)

	rb.Write([]byte("abc"))
	if !bytes.Equal(rb.Bytes(), []byte("abc")) {
		t.Errorf("expected abc, got %s", rb.Bytes())
	}

	rb.Write([]byte("defghij"))
	if !bytes.Equal(rb.Bytes(), []byte("abcdefghij")) {
		t.Errorf("expected abcdefghij, got %s", rb.Bytes())
	}

	rb.Write([]byte("k"))
	if !bytes.Equal(rb.Bytes(), []byte("bcdefghijk")) {
		t.Errorf("expected bcdefghijk, got %s", rb.Bytes())
	}

	rb.Write([]byte("lmn"))
	if !bytes.Equal(rb.Bytes(), []byte("efghijklmn")) {
		t.Errorf("expected efghijklmn, got %s", rb.Bytes())
	}

	rb.Write([]byte("012345678901234"))
	if !bytes.Equal(rb.Bytes(), []byte("5678901234")) {
		t.Errorf("expected 5678901234, got %s", rb.Bytes())
	}
}
