package utp

/*** UTP Quotation Messages (UQDF) ***/

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
	Orig MarketCenterOriginatorID
	// SubMarketId is the Sub Market Center ID.
	SubMarketId SubMarketCenterID
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
	// QuoteCond is the quote condition (byte).
	QuoteCond QuoteConditionCode
	// SipGenUpdate is the SIP generated update flag (byte).
	SipGenUpdate SipGeneratedUpdate
	// LuldBboIndicator is the LULD BBO Indicator (byte).
	LuldBboIndicator LluBboIndicator
	// Rii is the Retail Interest Indicator (byte).
	Rii RetailInterestCode
	// Nbbo is the NBBO Appendage Indicator (byte).
	Nbbo NbboAppendageIndicator
	// LuldNbboIndicator is the LULD National BBO Indicator (byte).
	LuldNbboIndicator LuldNbboIndicator
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
	QuoteCond QuoteConditionCode
	// SipGenUpdate is the SIP generated update flag.
	SipGenUpdate SipGeneratedUpdate
	// LuldBboIndicator is the LULD BBO Indicator.
	LuldBboIndicator LuldBboIndicator
	// Rii is the Retail Interest Indicator.
	Rii RetailInterestCode
	// Nbbo is the NBBO Appendage Indicator.
	Nbbo NbboAppendageIndicator
	// LuldNbboIndicator is the LULD National BBO Indicator.
	LuldNbboIndicator LuldNbboIndicator
	// FinraAdfMpidIndicator is the FINRA ADF MPID Appendage Indicator.
	FinraAdfMpidIndicator FinraAdfMpidAppendageIndicator
}

/** National BBO Appendage **/

type NbboAppendageShort struct {
	// NbboQuoteCond is the NBBO quote condition.
	NbboQuoteCond NbboQuoteConditionCode
	// NbBidMarketCenter is the national best bid market center.
	NbBidMarketCenter MarketCenterOriginatorID
	// NbBidPrice is the national best bid price.
	NbBidPrice uint16
	// NbBidSize is the national best bid size.
	NbBidSize uint16
	// NbAskMarketCenter is the national best ask market center.
	NbAskMarketCenter MarketCenterOriginatorID
	// NbAskPrice is the national best ask price.
	NbAskPrice uint16
	// NbAskSize is the national best ask size.
	NbAskSize uint16
}

type NbboAppendageLong struct {
	// NbboQuoteCond is the NBBO quote condition.
	NbboQuoteCond NbboQuoteConditionCode
	// NbBidMarketCenter is the national best bid market center.
	NbBidMarketCenter MarketCenterOriginatorID
	// NbBidPrice is the national best bid price.
	NbBidPrice uint64
	// NbBidSize is the national best bid size.
	NbBidSize uint32
	// NbAskMarketCenter is the national best ask market center.
	NbAskMarketCenter MarketCenterOriginatorID
	// NbAskPrice is the national best ask price.
	NbAskPrice uint64
	// NbAskSize is the national best ask size.
	NbAskSize uint32
}

// TODO: Where to put FinraAdfMpidAppendageIndicator?
type FinraAdfMpidAppendage struct {
	// BidAdfMpid is the bid ADF MPID.
	BidAdfMpid [4]byte
	// AskAdfMpid is the ask ADF MPID.
	AskAdfMpid [4]byte
}

type FinraAdfMpidQuotationMessage struct {
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
	QuoteCond QuoteConditionCode
	// Mpid is the FINRA Market Participant.
	Mpid [4]byte
}

/*** Trade Messages (UTDF) ***/

// TradeReportMessageShort is the short form of the quotation data message
// format.
// Expectations:
// 1) Bid and Ask Prices to have a max price of $655.35.
// 2) Bid and Ask Prices only use 2 decimal precision.
// 3) Bid and Ask Sizes are less than 65535.
// 4) Sale Condition modifier is not equal to "R" (Seller).
type TradeReportMessageShort struct {
	// Header is the message header.
	Header MessageHeader
	// Timestamp2 is the FINRA timestamp.
	Timestamp2 uint64
	// Symbol is the security identifier.
	Symbol [5]byte
	// TradeId is the trade Id.
	TradId uint64
	// Price is the trade price.
	Price uint16
	// Volume is the trade volume.
	Volume uint16
	// Cond is the sale condition.
	Cond [4]SaleConditionModifier
	// TradeThrExempt is the trade through exempt flag.
	TradeThrExempt byte
	// ConsPriceChangeInd is the consolidated price change indicator.
	ConsPriceChangeInd byte
	// PartPriceChangeInd is the participant price change indicator.
	PartPriceChangeInd byte
}

type TradeReportMessageLong struct {
	// Header is the message header.
	Header MessageHeader
	// Timestamp2 is the FINRA timestamp.
	Timestamp2 uint64
	// Symbol is the security identifier.
	Symbol [11]byte
	// TradeId is the trade Id.
	TradId uint64
	// Price is the trade price.
	Price uint64
	// Volume is the trade volume.
	Volume uint32
	// Cond is the sale condition.
	//
	// If equal to "R" (Seller), the SaleDays field will reflect the number of
	// dyas that may elapse before stock delivery.
	Cond [4]byte
	// TradeThrExempt is the trade through exempt flag.
	TradeThrExempt byte
	// SaleDays is the Seller's sale days.
	SaleDays uint16
	// ConsPriceChangeInd is the consolidated price change indicator.
	ConsPriceChangeInd byte
	// PartPriceChangeInd is the participant price change indicator.
	PartPriceChangeInd byte
}

type TradeCancelErrorMessage struct {
	// Header is the message header.
	Header MessageHeader
	// Timestamp2 is the FINRA timestamp.
	Timestamp2 uint64
	// Symbol is the security identifier.
	Symbol [11]byte
	// CancelType is the trade cancellation type.
	CancelType TradeCancelTypeCode
	// OrigTradeId is the original trade Id
	OrigTradeId uint64
	// OrigPrice is the original trade price
	OrigPrice uint64
	// OrigVolume is the original trade volume
	OrigVolume uint32
	// OrigCond is the original sale cond
	OrigCond [4]byte
	// OrigTradeThrExempt is the original trade through exempt flag.
	OrigTradeThrExempt byte
	// OrigSaleDays is the original Seller's sale days.
	OrigSaleDays uint16
	// ConsHighPrice is the consolidated high price.
	ConsHighPrice uint64
	// ConsLowPrice is the consolidated low price.
	ConsLowPrice uint64
	// ConsLastPrice is the consolidated last price.
	ConsLastPrice uint64
	// ConsVolume is the consolidated volume.
	ConsVolume uint64
	// ConsPriceChangeInd is the consolidated price change indicator.
	ConsPriceChangeInd byte
	// ConsLastPriceOrig is the market center originator ID.
	ConsLastPriceOrig byte
	// PartHighPrice is the market participant high price.
	PartHighPrice uint64
	// PartLowPrice is the market participant low price.
	PartLowPrice uint64
	// PartLastPrice is the market participant last price.
	PartLastPrice uint64
	// PartVolume is the market participant volume.
	PartVolume uint64
}

type TradeCorrectionMessage struct {
	// Header is the message header.
	Header MessageHeader
	// Timestamp2 is the FINRA timestamp.
	Timestamp2 uint64
	// Symbol is the security identifier.
	Symbol [11]byte
	// OrigTradeId is the original trade Id
	OrigTradeId uint64
	// OrigPrice is the original trade price
	OrigPrice uint64
	// OrigVolume is the original trade volume
	OrigVolume uint32
	// OrigCond is the original sale cond
	OrigCond [4]byte
	// OrigTradeThrExempt is the original trade through exempt flag.
	OrigTradeThrExempt byte
	// OrigSaleDays is the original Seller's sale days.
	OrigSaleDays uint16
	// CorrTradeId is the corrected trade Id
	CorrTradeId uint64
	// CorrPrice is the corrected trade price
	CorrPrice uint64
	// CorrVolume is the corrected trade volume
	CorrVolume uint32
	// CorrCond is the corrected sale cond
	CorrCond [4]byte
	// CorrTradeThrExempt is the corrected trade through exempt flag.
	CorrTradeThrExempt byte
	// CorrSaleDays is the corrected Seller's sale days.
	CorrSaleDays uint16
	// ConsHighPrice is the consolidated high price.
	ConsHighPrice uint64
	// ConsLowPrice is the consolidated low price.
	ConsLowPrice uint64
	// ConsLastPrice is the consolidated last price.
	ConsLastPrice uint64
	// ConsVolume is the consolidated volume.
	ConsVolume uint64
	// ConsPriceChangeInd is the consolidated price change indicator.
	ConsPriceChangeInd byte
	// ConsLastPriceOrig is the market center originator ID.
	ConsLastPriceOrig byte
	// PartHighPrice is the market participant high price.
	PartHighPrice uint64
	// PartLowPrice is the market participant low price.
	PartLowPrice uint64
	// PartLastPrice is the market participant last price.
	PartLastPrice uint64
	// PartVolume is the market participant volume.
	PartVolume uint64
}

type PriorDayAsOrTradeMessage struct {
	// Header is the message header.
	Header MessageHeader
	// Timestamp2 is the FINRA timestamp.
	Timestamp2 uint64
	// Symbol is the security identifier.
	Symbol [11]byte
	// TradeId is the trade Id.
	TradId uint64
	// Price is the trade price.
	Price uint64
	// Volume is the trade volume.
	Volume uint32
	// Cond is the sale condition.
	Cond [4]byte
	// TradeThrExempt is the trade through exempt flag.
	TradeThrExempt byte
	// SaleDays is the Seller's sale days.
	SaleDays uint16
	// AsOfAction is the as-of action.
	AsOfAction AsOfAction
	// PriorTime is the timestamp of trade
	PriorTime uint64
}

/*** Administrative Messages ***/

/** Trade and Quote Services (UQDF & UTDF) **/

type GeneralAdministrativeMessage struct {
	// Header is the message header.
	Header MessageHeader
	// TextLen is the text length.
	TextLen uint16
	// Text is the text.
	Text []byte
}

type CrossSroTradingActionMessage struct {
	// Header is the message header.
	Header MessageHeader
	// Symbol is the security identifier.
	Symbol [11]byte
	// Action is the trading action code.
	Action byte
	// ActionSequence is the trading action sequence number.
	ActionSequence uint32
	// ActionTime is the timestamp of when the action occurred.
	ActionTime uint64
	// Reason is the reason for the trading action.
	Reason [6]byte
}
