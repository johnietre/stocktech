package soupbintcp

import (
	"fmt"

	utils "github.com/johnietre/utils/go"
)

type DataStore interface {
	Get(SequenceNumber) ([]byte, error)
	Set(SequenceNumber, []byte) error
}

type SliceDataStore struct {
	start uint64
	data  *utils.RWMutex[[][]byte]
}

func NewSliceDataStore(startSeqNum uint64) *SliceDataStore {
	return &SliceDataStore{
		start: startSeqNum,
		data:  utils.NewRWMutex([][]byte{}),
	}
}

var (
	ErrNotFound = fmt.Errorf("not found")
)

func (sds *SliceDataStore) Get(seqNum SequenceNumber) (data []byte, err error) {
	sn, ok := seqNum.ToUint64Safe()
	if !ok || sn < sds.start {
		return nil, ErrNotFound
	}
	i := sn - sds.start
	sds.data.RApply(func(sp *[][]byte) {
		s := *sp
		if i < uint64(len(s)) {
			data = s[i]
		} else {
			err = ErrNotFound
		}
	})
	return
}

func (sds *SliceDataStore) Set(seqNum SequenceNumber, data []byte) error {
	sds.data.Apply(func(sp *[][]byte) {
		*sp = append(*sp, data)
	})
	return nil
}

type MapDataStore struct {
	data *utils.SyncMap[SequenceNumber, []byte]
}

func NewMapDataStore() *MapDataStore {
	return &MapDataStore{
		data: utils.NewSyncMap[SequenceNumber, []byte](),
	}
}

func (mds *MapDataStore) Get(seqNum SequenceNumber) ([]byte, error) {
	payload, ok := mds.data.Load(seqNum)
	if !ok {
		return nil, ErrNotFound
	}
	return payload, nil
}

var (
	ErrDuplicate = fmt.Errorf("duplicate sequence number")
)

func (mds *MapDataStore) Set(seqNum SequenceNumber, data []byte) error {
	if utils.Second(mds.data.LoadOrStore(seqNum, data)) {
		return ErrDuplicate
	}
	return nil
}
