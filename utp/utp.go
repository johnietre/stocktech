package utp

// MessageHeader is the header that precedes each message specific data
// section.
type MessageHeader struct {
	// Version is the protocol version.
	Version byte
	// MsgCategory is the message category.
	MsgCategory byte
	// MsgType is the message type.
	MsgType byte
	// Orig is the Market Center Originator ID.
	Orig byte
	// SubMarketId is the Sub Market Center ID.
	SubMarketId byte
	// SipTime is the SIP timestamp.
	SipTime uint64
	// Timestamp1 is the participant timestamp.
	Timestamp1 uint64
	// PartToken is the participant token.
	PartToken uint64
}

// QuoteMessageShort is the short form of the quotation data message format.
// Expectations:
// 1) Bid and Ask Prices to have a max price of $655.35.
// 2) Bid and Ask Prices only use 2 decimal precision.
// 3) Bid and Ask Sizes are less than 65535.
// 4) Quote is not a FINRA quote.
type QuoteMessageShort struct {
	// Header is the message header.
	Header MessageHeader
	// Symbol is the security identifier.
	Symbol [5]byte
	// BidPrice is the bid price.
	BidPrice uint16
	// BidSize is the bid size.
	BidSize uint16
	// AskPrice is the ask price.
	AskPrice uint16
	// AskSize is the ask size.
	AskSize uint16
	// QuoteCond is the quote condition.
	QuoteCond byte
	// SipGenUpdate is the SIP generated update flag.
	SipGenUpdate byte
	// LuldBboIndicator is the LULD BBO Indicator.
	LuldBboIndicator byte
	// Rii is the Retail Interest Indicator.
	Rii byte
	// Nbbo is the NBBO Appendage Indicator.
	Nbbo byte
	// LuldNbboIndicator is the LULD National BBO Indicator.
	LuldNbboIndicator byte
}

type QuoteMessageLong struct {
	// Header is the message header.
	Header MessageHeader
	// Timestamp2 is the FINRA timestamp.
	Timestamp2 uint64
	// Symbol is the security indentifier.
	Symbol [11]byte
	// BidPrice is the bid price.
	BidPrice uint64
	// BidSize is the bid size.
	BidSize uint32
	// AskPrice is the ask price.
	AskPrice uint64
	// AskSize is the ask size.
	AskSize uint32
	// QuoteCond is the quote condition.
	QuoteCond byte
	// SipGenUpdate is the SIP generated update flag.
	SipGenUpdate byte
	// LuldBboIndicator is the LULD BBO Indicator.
	LuldBboIndicator byte
	// Rii is the Retail Interest Indicator.
	Rii byte
	// Nbbo is the NBBO Appendage Indicator.
	Nbbo byte
	// LuldNbboIndicator is the LULD National BBO Indicator.
	LuldNbboIndicator byte
	// FinraAdfMpidIndicator is the FINRA ADF MPID Appendage Indicator.
	FinraAdfMpidIndicator byte
}

type NbboAppendageShort struct {
	// NbboQuoteCond is the NBBO quote condition.
	NbboQuoteCond     byte
	NbBidMarketCenter byte
	// NbBidPrice is the national best bid price.
	NbBidPrice uint16
	// NbBidSize is the national best bid size.
	NbBidSize         uint16
	NbAskMarketCenter byte
	// NbAskPrice is the national best ask price.
	NbAskPrice uint16
	// NbAskSize is the national best ask size.
	NbAskSize uint16
}

type NbboAppendageLong struct {
	// NbboQuoteCond is the NBBO quote condition.
	NbboQuoteCond     byte
	NbBidMarketCenter byte
	// NbBidPrice is the national best bid price.
	NbBidPrice uint64
	// NbBidSize is the national best bid size.
	NbBidSize         uint32
	NbAskMarketCenter byte
	// NbAskPrice is the national best ask price.
	NbAskPrice uint64
	// NbAskSize is the national best ask size.
	NbAskSize uint32
}

type FinraAdfMpidAppendage struct {
	// BidAdfMpid is the bid ADF MPID.
	BidAdfMpid [4]byte
	// AskAdfMpid is the ask ADF MPID.
	AskAdfMpid [4]byte
}

type FinraAdfMpidQuoteMessage struct {
	// Header is the message ehader
	Header MessageHeader
	// Timestamp2 is the FINRA timestamp.
	Timestamp2 uint64
	// Symbol is the security indentifier.
	Symbol [11]byte
	// BidPrice is the bid price.
	BidPrice uint64
	// BidSize is the bid size.
	BidSize uint32
	// AskPrice is the ask price.
	AskPrice uint64
	// AskSize is the ask size.
	AskSize uint32
	// QuoteCond is the quote condition.
	QuoteCond byte
	// Mpid is the FINRA Market Participant.
	Mpid [4]byte
}
