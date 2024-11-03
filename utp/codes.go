package utp

// TODO: Where byte is an ASCII number, check to see if it's actually supposed
// to be an ASCII number or the literal numer (e.g., '4' vs 4 (which is a
// control character)).

type MarketCenterOriginatorID = byte

const (
	// Mcoi_CboeBYX represents Cboe BYX Exchange, Inc.
	Mcoi_CboeBYX MarketCenterOriginatorID = 'Y'
	// Mcoi_CboeBZX represents Cboe BZX Exchange, Inc.
	Mcoi_CboeBZX MarketCenterOriginatorID = 'Z'
	// Mcoi_CboeEDGA represents Cboe EDGA Exchange, Inc.
	Mcoi_CboeEDGA MarketCenterOriginatorID = 'J'
	// Mcoi_CboeEDGX represents Cbox EDGX Exchange, Inc.
	Mcoi_CboeEDGX MarketCenterOriginatorID = 'K'
	// Mcoi_Cboe represents Cboe Exchange, Inc.
	Mcoi_Cboe MarketCenterOriginatorID = 'W'

	// Mcoi_NasdaqBX represents Nasdaq BX, Inc.
	Mcoi_NasdaqBX MarketCenterOriginatorID = 'B'
	// Mcoi_NasdaqPHLX represents Nasdaq PHLX LLC.
	Mcoi_NasdaqPHLX MarketCenterOriginatorID = 'X'
	// Mcoi_Nasdaq represents Nasdaq, Inc.
	Mcoi_Nasdaq MarketCenterOriginatorID = 'Q'
	// Mcoi_NasdaqISE represents Nasdaq ISE, LLC.
	Mcoi_NasdaqISE MarketCenterOriginatorID = 'I'

	// Mcoi_Nyse represents New York Stock Exchange LLC.
	Mcoi_Nyse MarketCenterOriginatorID = 'N'
	// Mcoi_NyseArca represents NYSE Arca, Inc.
	Mcoi_NyseArca MarketCenterOriginatorID = 'P'
	// Mcoi_NyseAmerican represents NYSE American, LLC.
	Mcoi_NyseAmerican MarketCenterOriginatorID = 'A'
	// Mcoi_NyseNational represents NYSE National, Inc.
	Mcoi_NyseNational MarketCenterOriginatorID = 'C'
	// Mcoi_NyseChicago represents NYSE Chicago, Inc.
	Mcoi_NyseChicago MarketCenterOriginatorID = 'M'

	// Mcoi_Finra represents Financial Industry Regulatory Authority (FINRA).
	Mcoi_Finra MarketCenterOriginatorID = 'D'
	// Mcoi_Iex represents Investor's Exchange LLC (IEX).
	Mcoi_Iex MarketCenterOriginatorID = 'V'
	// Mcoi_Ltse represents Long-Term Stock Exchange (LTSE).
	Mcoi_Ltse MarketCenterOriginatorID = 'L'
	// Mcoi_Miax represents MIAX Pearl, LLC (MIAX).
	Mcoi_Miax MarketCenterOriginatorID = 'H'
	// Mcoi_Memx represents MEMX LLC (MEMX).
	Mcoi_Memx MarketCenterOriginatorID = 'U'

	// Mcoi_MarketIndependent represents Market Independent (generated by SIP).
	Mcoi_MarketIndependent MarketCenterOriginatorID = 'E'
)

type SubMarketCenterID = byte

const (
	// Smci_FinraNyseTrf represents FINRA / NYSE TRF.
	Smci_FinraNyseTrf SubMarketCenterID = 'N'
	// Smci_FinraNyseTrfCarteret represents FINRA / NYSE TRF Carteret.
	Smci_FinraNyseTrfCarteret SubMarketCenterID = 'Q'
	// Smci_FinraNyseTrfChicago represents FINRA / NYSE TRF Chicago.
	Smci_FinraNyseTrfChicago SubMarketCenterID = 'B'
	// SmicFinraADF represents a trade originating from FINRA Alternative Display
	// Facility.
	Smci_FinraADF SubMarketCenterID = ' '
)

// QuoteConditionCode represents an allowable value at the market center level.
type QuoteConditionCode = byte

const (
	// Qc_ManualAskAutoBid represents manual ask, automated bid.
	Qc_ManualAskAutoBid QuoteConditionCode = 'A'
	// Qc_ManualBidAutoAsk represents manual bid, automated ask.
	Qc_ManualBidAutoAsk QuoteConditionCode = 'B'
	// Qc_FastTrading represents fast trading.
	Qc_FastTrading QuoteConditionCode = 'F'
	// Qc_ManualBidAsk represents manual bid and ask.
	Qc_ManualBidAsk QuoteConditionCode = 'H'
	// Qc_OrderImbalance represents order imbalance.
	Qc_OrderImbalance QuoteConditionCode = 'I'
	// Qc_CloseQuote represents closed quote.
	Qc_CloseQuote QuoteConditionCode = 'L'
	// Qc_NonFirmQuote represents non-firm quote.
	Qc_NonFirmQuote QuoteConditionCode = 'N'
	// Qc_OpeningQuoteAutomated represents opening quote automated.
	Qc_OpeningQuoteAutomated QuoteConditionCode = 'O'
	// Qc_Reg2sOpenQuoteAutomated represents regular, two-sided open quote
	// automated.
	Qc_Reg2sOpenQuoteAutomated QuoteConditionCode = 'R'
	// Qc_ManualBidAskNonFirm represents manual bid and ask (non-firm).
	Qc_ManualBidAskNonFirm QuoteConditionCode = 'U'
	// Qc_OrderInflux represents order influx.
	Qc_OrderInflux QuoteConditionCode = 'X'
	// Qc_AutoNoBidOffer represents automated bid, no offer; or automated
	// offer, no bid.
	Qc_AutoNoBidOffer QuoteConditionCode = 'Y'
	// Qc_NoOpenResume represents no open/no resume.
	Qc_NoOpenResume QuoteConditionCode = 'Z'
	// Qc_IntradayAuction represents intraday auction.
	Qc_IntradayAuction QuoteConditionCode = '4'
)

// TODO: Should this be part of QuoteConditionCode?
type NbboQuoteConditionCode = byte

const (
	// Nqc_Closed represents NBBO Closed.
	Nqc_Closed QuoteConditionCode = 'L'
	// Nqc_Reg2sOpen represents NBBO Regular, two-sided open.
	Nqc_Reg2sOpen QuoteConditionCode = 'R'
	// Nqc_Reg1sOpen represents NBBO Regular, one-sided open.
	Nqc_Reg1sOpen QuoteConditionCode = 'Y'
)

type SipGeneratedUpdate = byte

const (
	// Sgu_MpidOrig represents a transaction which originated from the market
	// participant identified in the Market Center Originator ID field of the
	// Message Header.
	Sgu_MpidOrig SipGeneratedUpdate = ' '
	// Sgu_SipGenTx represents a message which is the result of a SIP-generated
	// transaction.
	Sgu_SipGenTx SipGeneratedUpdate = 'E'
)

type LuldBboIndicator = byte

const (
	// Lb_NA represents Limit Up Limit Down not applicable.
	Lb_NA LuldBboIndicator = ' '
	// Lb_BidNE represents bid price above upper limit price band - bid is
	// non-executable.
	Lb_BidNE LuldBboIndicator = 'A'
	// Lb_AskNE represents ask price below lower limit price band - ask is
	// non-executable.
	Lb_AskNE LuldBboIndicator = 'B'
	// Lb_NE represents bid and ask outside price band. Not executable.
	Lb_NE LuldBboIndicator = 'C'
)

type LuldNbboIndicator = byte

const (
	// Ln_NA represents Limit Up Limit Down not applicable
	Ln_NA LuldNbboIndicator = ' '
	// TODO
)

// Retail Interest Code
type RetailInterestIndicator = byte

const (
	// Rii_NA represents retail interest not applicable.
	Rii_NA RetailInterestIndicator = ' '
	// Rii_BidQuote represents retail interest on bid quote.
	Rii_OnBidQuote RetailInterestIndicator = 'A'
	// Rii_AskQuote represents retail interest on ask quote.
	Rii_OnAskQuote RetailInterestIndicator = 'B'
	// Rii_BidAskQuote represents retail interest on both bid and ask quote.
	Rii_OnBidAskQuote RetailInterestIndicator = 'C'
)

type NbboAppendageIndicator = byte

const (
	// Nai_NoChange represents no National BBO change.
	Nai_NoChange NbboAppendageIndicator = '0'
	// Nai_NoCalc represents no National BBO can be calculated.
	Nai_NoCalc NbboAppendageIndicator = '1'
	// Nai_Short represents short form National BBO appendage attached.
	Nai_Short NbboAppendageIndicator = '2'
	// Nai_Long represents long form National BBO appendage attached.
	Nai_Long NbboAppendageIndicator = '3'
	// Nai_Quote represents quote contains all National BBO information.
	Nai_Quote NbboAppendageIndicator = '4'
)

type FinraAdfMpidAppendageIndicator = byte

const (
	// Famai_NA represents not applicable.
	Famai_NA FinraAdfMpidAppendageIndicator = ' '
	// Famai_NoChanges represents no ADF MPID changes.
	Famai_NoChanges FinraAdfMpidAppendageIndicator = '0'
	// Famai_NotExist represents no ADF MPID exists.
	Famai_NotExist FinraAdfMpidAppendageIndicator = '1'
	// Famai_Attached represents ADF MPID(s) attached.
	Famai_Attached FinraAdfMpidAppendageIndicator = '2'
)

type ConsolidatedPriceChangeIndicator = byte

const ()

type ParticipantPriceChangeIndicator = byte

const ()

type RegShoActionCode = byte

const (
	// Rsa_NoTest represents no price test in effect.
	Rsa_NoTest RegShoActionCode = '0'
	// Rsa_InEffect represents Reg SHO in effect due to an intra day price drop
	// in security.
	Rsa_InEffect RegShoActionCode = '0'
	// Rsa_Restriction represents Reg SHO restriction remains in effect.
	Rsa_Restriction RegShoActionCode = '0'
)

type TradeThroughExemptCode = byte

const (
	// Tte_TTE represents trade through exempt.
	Tte_TTE TradeThroughExemptCode = 'X'
	// Tte_Not611TTE represents not 611 trade through exempt.
	Tte_Not611TTE TradeThroughExemptCode = ' '
)

type SaleConditionModifier = byte

const (
	// Scm_RegularSale represents regular sale.
	Scm_RegularSale = '@'
	// Scm_Acquisition represents acquisition.
	Scm_Acquisition = 'A'
	// Scm_BunchedTrade represents bunched trade.
	Scm_BunchedTrade = 'B'
	// Scm_CashSale represents cash sale.
	Scm_CashSale = 'C'
	// Scm_Distribution represents distribution.
	Scm_Distribution = 'D'
	// Scm_Placeholder represents placeholder.
	Scm_Placeholder = 'E'
	// Scm_IntermarketSweep represents intermarket sweep.
	Scm_IntermarketSweep = 'F'
	// Scm_BunchedSoldTrade represents sold trade.
	Scm_BunchedSoldTrade = 'G'
	// Scm_PriceVariationTrade represents variation trade.
	Scm_PriceVariationTrade = 'H'
	// Scm_OddLotTrade represents odd lot trade.
	Scm_OddLotTrade = 'I'
	// Scm_Rule155Trade represents rule 155 trade (AMEX).
	Scm_Rule155Trade = 'K'
	// Scm_SoldLast represents sold last.
	Scm_SoldLast = 'L'
	// Scm_MarketCenterOfficialClose represents market center official close.
	Scm_MarketCenterOfficialClose = 'M'
	// Scm_NextDay represents next day.
	Scm_NextDay = 'N'
	// Scm_OpeningPrints represents opening prints.
	Scm_OpeningPrints = 'O'
	// Scm_PriorReferencePrice represents prior reference price.
	Scm_PriorReferencePrice = 'P'
	// Scm_MarketCenterOfficialOpen represents market center official open.
	Scm_MarketCenterOfficialOpen = 'Q'
	// Scm_Seller represents seller.
	Scm_Seller = 'R'
	// Scm_SplitTrade represents split trade.
	Scm_SplitTrade = 'S'
	// Scm_FormT represents form T.
	Scm_FormT = 'T'
	// Scm_ExtendedTradingHours represents extended trading hours.
	Scm_ExtendedTradingHours = 'U'
	// Scm_ContingentTrade represents contingent trade.
	Scm_ContingentTrade = 'V'
	// Scm_AveragePriceTrade represents average price trade.
	Scm_AveragePriceTrade = 'W'
	// Scm_CrossPeriodicAuctionTrade represents cross/periodic auction trade.
	Scm_CrossPeriodicAuctionTrade = 'X'
	// Scm_YellowFlagRegularTrade represents yellow flag regular trade.
	Scm_YellowFlagRegularTrade = 'Y'
	// Scm_Sold represents sold (out of sequence).
	Scm_Sold = 'Z'
	// Scm_StoppedStock represents stopped stock (regular trade).
	Scm_StoppedStock = '1'
	// Scm_DerivativelyPriced represents derivatively priced.
	Scm_DerivativelyPriced = '4'
	// Scm_ReOpeningPrints represents opening prints.
	Scm_ReOpeningPrints = '5'
	// Scm_ClosingPrints represents closing prints.
	Scm_ClosingPrints = '6'
	// Scm_QualifiedContingentTrade qualified contingent trade ("QCT").
	Scm_QualifiedContingentTrade = '7'
	// Scm_PlaceholderFor611Exempt represents placeholder for 611 exempt.
	Scm_PlaceholderFor611Exempt = '8'
	// Scm_CorrectedConsolidatedClose represents corrected consolidated close
	// (per listing market).
	Scm_CorrectedConsolidatedClose = '9'
)

type TradeCancelTypeCode = byte

const (
	Tct_Cancel TradeCancelTypeCode = 'C'
	Tct_Error  TradeCancelTypeCode = 'E'
)

type AsOfAction = byte

const (
	Ao_TradeAddition AsOfAction = 'A'
	Ao_TradeCancel   AsOfAction = 'C'
)

// TODO: Is this the desired type?
type TradingActionReasonCode = [4]byte

const ()

type TradingActionCode = byte

const (
	Ta_TradingHalt            TradingActionCode = 'H'
	Ta_QuotationResumption    TradingActionCode = 'Q'
	Ta_TradingResumption      TradingActionCode = 'T'
	Ta_VolatilityTradingPause TradingActionCode = 'P'
)
