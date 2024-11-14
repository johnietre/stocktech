use std::cmd::{PartialEq, PartialOrd};
use std::fmt;

pub const REVISION: u8 = 4;

/* Inbound messages */

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Symbol([u8; 8]);

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Price(u64);

impl Price {
    pub const MAX: Self = Self(199_999_9900);
    pub const MAX_F64: f64 = 199_999.9900;
    pub const MAX_U64: u64 = 199_999_9900;
    pub const MARKET: Self = Self(200_000_0000);
    pub const MARKET_CROSS: Self = Self(214_748_3647);

    pub fn from_f64(f: f64) -> Option<Self> {
        if f > Self::MAX_F64 || f < 0.0 {
            return None;
        }
        Some(Self(f as u64 * 10_000 + ((f.fract() * 10_000.0))))
    }

    pub fn to_f64(self) -> f64 {
        // NOTE: max is representable as f64
        self.0 as f64 / 10_000
    }

    pub fn to_f64_opt(self) -> Option<f64> {
        if self.0 <= Self::MAX_U64 {
            Some(self.0 as f64 / 10_000)
        } else {
            None
        }
    }

    pub fn to_parts(self) -> (u32, u32) {
        (self.0 / 10_000, self.0 % 10_000)
    }

    pub const fn is_market(self) -> bool {
        // TODO: Check for market cross too?
        self == Self::MARKET || self == Self::MARKET_CROSS || self.0 == u64::MAX
    }
}

impl PartialEq for Price {
    fn eq(&self, other: &Self) -> bool {
        if self.0 <= Self::MAX_U64 {
            self.0 == other.0
        } else {
            false
        }
    }
}

impl PartialOrd for Price {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.0 <= Self::MAX_U64 && other <= Self::MAX_U64 {
            self.0.partial_cmp(&other.0)
        } else {
            None
        }
    }
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_market() {
            // FIXME: what to do
            write!(f, "MARKET ORDER")
        } else {
            let (dollars, cents) = self.to_parts();
            write!(f, "{dollars}.{cents:04}");
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct SignedPrice(i64);

impl SignedPrice {
    pub const MIN: Self = Self(-199_999_9900);
    pub const MIN_F64: f64 = -199_999.9900;
    pub const MAX: Self = Self(199_999_9900);
    pub const MAX_F64: f64 = 199_999.9900;
    pub const MARKET: Self = Self(200_000_0000);
    pub const MARKET_CROSS: Self = Self(214_748_3647);

    pub fn from_f64(f: f64) -> Option<Self> {
        if f > Self::MAX_F64 || f < Self::MIN_F64 {
            None
        } else if f >= 0.0 {
            Some(Self(f as i64 * 10_000 + (f.fract() as i64 * 10_000)))
        } else {
            Some(Self(f as i64 * 10_000 - (f.fract() as i64 * 10_000)))
        }
    }

    pub fn to_f64(self) -> f64 {
        // NOTE: max is representable as f64
        self.0 as f64 / 10_000
    }

    pub const fn is_market(self) -> bool {
        // TODO: Check for market cross too?
        self == Self::MARKET || self == Self::MARKET_CROSS || self.0 == i64::MAX
    }
}

#[repr(u8)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug)]
pub enum OrderSide {
    Buy = b'B',
    Sell = b'S',
    SellShort = b'T',
    SellShortExempt = b'E',
}

// TODO
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct UserRefNum(u32);

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ClOrdId([u8; 14]);

impl ClOrdId {
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 14 {
            return None;
        }
        let mut arr = [0u8; 14];
        for i in 0..bytes.len() {
            let b = bytes[i];
            // FIXME: are there specific places spaces are/aren't allowed e.g., are spaces only
            // padding characers, can an ID be all spaces?
            if !b.is_ascii_alphanumeric() || b == b' ' {
                return None;
            }
            arr[i] = b;
        }
        Some(Self(arr))
    }
}

impl fmt::Display for ClOrdId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write(f, "{}", String::from_utf8_lossy(&self.0))
    }
}

#[repr(u8)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug)]
pub enum TimeInForce {
    Day = b'0',
    IOC = b'3',
    GTX = b'5',
    GTT = b'6',
    AfterHours = b'E',
}

#[repr(u8)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug)]
pub enum Display {
    Visible = b'Y',
    Hidden = b'N',
    Attributable = b'A',
}

#[repr(u8)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug)]
pub enum Capacity {
    Agency = b'A',
    Principal = b'P',
    Riskless = b'R',
    Other = b'O',
}

#[repr(u8)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug)]
pub enum InterMarketSweepEligibility {
    Eligible = b'Y',
    NotEligible = b'N',
}
pub type IMSE = InterMarketSweepEligibility;

#[repr(u8)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug)]
pub enum CrossType {
    ContinuousMarket = b'N',
    OpeningCross = b'O',
    ClosingCross = b'C',
    HaltIpo = b'H',
    Supplemental = b'S',
    Retail = b'R',
    ExtendedLife = b'E',
    AfterHoursClose = b'A',
}

#[repr(u8)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug)]
pub enum CustomerType {
    // TODO: default?
    RetailDesignatedOrder = b'R',
    NotRetailDesignated = b'N',
}

#[repr(u8)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PriceType {
    #[default]
    Limit = b'L',
    MarketPeg = b'P',
    MidpointPeg = b'M',
    PrimaryPeg = b'R',
    MarketMakerPeg = b'Q',
    Midpoint = b'm',
}

#[repr(u8)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PostOnly {
    PostOnly = b'P',
    #[default]
    No = b'N',
}

#[repr(u8)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug, Default)]
pub enum HandleInst {
    No = b' ',
    // TODO
}

#[repr(u8)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug)]
pub enum BboWeightIndicator {
    // TODO
}

// TODO: Discretion Price type?

#[repr(u8)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug)]
pub enum OptionTag {
    SecondaryOrdRefNum = 1,
    Firm = 2,
    MinQty = 3,
    CustomerType = 4,
    MaxFloor = 5,
    PriceType = 6,
    PegOffset = 7,
    DiscretionPrice = 9,
    DiscretionPriceType = 10,
    DiscretionPegOffset = 11,
    PostOnly = 12,
    RandomReserves = 13,
    Route = 14,
    ExpireTime = 15,
    TradeNow = 16,
    HandleInst = 17,
    BboWeightIndicator = 18,
    DisplayQuantity = 22,
    DisplayPrice = 23,
    GroupId = 24,
    SharesLocated = 25,
    LocateBroker = 26,
    Side = 27,
    UserRefIdx = 28,
}

impl OptionTag {
    pub fn size(self) -> usize {
        match self {
            OptionTag::SecondaryOrdRefNum => 8,
            OptionTag::Firm => 4,
            OptionTag::MinQty => 4,
            OptionTag::CustomerType => 1,
            OptionTag::MaxFloor => 4,
            OptionTag::PriceType => 1,
            OptionTag::PegOffset => 4,
            OptionTag::DiscretionPrice => 8,
            OptionTag::DiscretionPriceType => 1,
            OptionTag::DiscretionPegOffset => 4,
            OptionTag::PostOnly => 1,
            OptionTag::RandomReserves => 4,
            OptionTag::Route => 4,
            OptionTag::ExpireTime => 4,
            OptionTag::TradeNow => 1,
            OptionTag::HandleInst => 1,
            OptionTag::BboWeightIndicator => 1,
            OptionTag::DisplayQuantity => 4,
            OptionTag::DisplayPrice => 8,
            OptionTag::GroupId => 2,
            OptionTag::SharesLocated => 1,
            OptionTag::LocateBroker => 4,
            OptionTag::Side => 1,
            OptionTag::UserRefIdx => 1,
        }
    }
}

#[repr(transparent)]
pub struct OptionValue(InnerOptionValue);
pub union InnerOptionValue {
    // TODO: supposed to be SecondaryOrderRefNum
    uint64: u64,
    firm: Firm,
    uint32: u32,
    price_type: PriceType,
    price: Price,
    signed_price: SignedPrice,
    post_only: PostOnly,
    // True is yes and false if no (obv)
    trade_now: bool,
}

impl OptionValue {
}

pub struct TagValue {
    length: u8,
    option_tag: OptionTag,
    option_value: OptionValue, // TODO
}

impl TagValue {
    pub fn new(tag: OptionTag, value: OptionValue) -> Result<Self, (OptionTag, OptionValue)> {
    }
}

// TODO
pub struct Firm([u8; 4]);

pub struct EnterOrder {
    pub user_ref_num: UserRefNum,
    pub side: OrderSide,
    pub quantity: u32,
    pub symbol: Symbol,
    pub price: Price,
    pub time_in_force: TimeInForce,
    pub display: Display,
    pub capacity: Capacity,
    pub inter_market_sweep_eligibility: InterMarketSweepEligibility,
    pub cross_type: CrossType,
    pub cl_ord_id: ClOrdId,
    //pub appendage_length: u16,
    pub optional_appendage: Vector<TagValue>,
}

impl EnterOrder {
    pub const TYPE: u8 = b'O';
}

pub struct ReplaceOrderRequest {
    pub orig_user_ref_num: UserRefNum,
    pub user_ref_num: UserRefNum,
    pub quantity: u32,
    pub price: Price,
    pub time_in_force: TimeInForce,
    pub display: Display,
    pub inter_market_sweep_eligibility: InterMarketSweepEligibility,
    pub cl_ord_id: ClOrdId,
    //pub appendage_length: u16,
    pub optional_appendage: Vector<TagValue>,
}

impl ReplaceOrderRequest {
    pub const TYPE: u8 = b'U';
}

pub struct CancelOrderRequest {
    pub user_ref_num: UserRefNum,
    pub quantity: u32,
    //pub appendage_length: u16,
    pub optional_appendage: Vector<TagValue>,
}

impl CancelOrderRequest {
    pub const TYPE: u8 = b'X';
}

pub struct ModifyOrderRequest {
    pub user_ref_num: UserRefNum,
    pub side: Side,
    pub quantity: u32,
    //pub appendage_length: u16,
    pub optional_appendage: Vector<TagValue>,
}

impl ModifyOrderRequest {
    pub const TYPE: u8 = b'M';
}

pub struct MassCancerRequest {
    pub user_ref_num: UserRefNum,
    pub firm: Firm,
    pub symbol: Symbol,
    //pub appendage_length: u16,
    pub optional_appendage: Vector<TagValue>,
}

impl MassOrderRequest {
    pub const TYPE: u8 = b'C';
}

pub struct DisableOrderEntryRequest {
    pub user_ref_num: UserRefNum,
    pub firm: Firm,
    //pub appendage_length: u16,
    pub optional_appendage: Vector<TagValue>,
}

impl DisableOrderEntryRequest {
    pub const TYPE: u8 = b'D';
}

pub struct EnableOrderEntryRequest {
    pub user_ref_num: UserRefNum,
    pub firm: Firm,
    //pub appendage_length: u16,
    pub optional_appendage: Vector<TagValue>,
    //pub appendage_length: u16,
    pub optional_appendage: Vector<TagValue>,
}

impl EnableOrderEntryRequest {
    pub const TYPE: u8 = b'E';
}

pub struct AccountQueryRequest {
    //pub appendage_length: u16,
    pub optional_appendage: Vector<TagValue>,
}

impl AccountQueryRequest {
    pub const TYPE: u8 = b'Q';
}

/* Outbound messages */

#[repr(u8)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug)]
pub enum EventCode {
    StartOfDay = b'S',
    EndOfDay = b'E',
}

#[repr(u8)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug)]
pub enum OrderState {
    Live = b'L',
    Dead = b'D',
}

#[repr(u8)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug)]
pub enum OrderCancelReason {
    // TODO
}

#[repr(u8)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug)]
pub enum LiquidityFlag {
    Added = b'A',
    ClosingCross = b'C',
    // Retail designated execution that added displayed liquidity
    RDETADL = b'e',
    HaltIpoCross = b'H',
    AfterHoursClosingCross = b'i',
    NonDisplayedAddingLiquidity = b'J',
    RpiOrderProvidesLiquidity = b'j',
    HaltCross = b'K',
    ClosingCrossImbalanceOnly = b'L',
    OpeningCrossImbalanceOnly = b'M',
    // Removed liquidity at a midpoint
    RemovedLiquidity = b'm',
    // Midpoint extended life order execution
    MidpointExtended = b'n',
    OpeningCross = b'O',
    // Removed price improving non-displayed liquidity
    RemovedPrice = b'p',
    // RMO retail order removes non-RPI midpoint liquidity
    RmoRetailOrder = b'q',
    Removed = b'R',
    // Retail Order removes RPI liquidity
    RetailOrderRemovesRpi = b'r',
    RetailOrderRemovesPrice = b't',
    // Added non-displayed liquidity via a Reserve order
    AddedNonDisplayedLiquidity = b'u',
    SupplementalOrderExecution = b'0',
    // TODO: rest and fix names/comments
}

#[repr(u8)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug)]
pub enum BrokenReason {
    Erroneuos = b'E',
    Consent = b'C',
    Supervisory = b'S',
    External = b'X',
}

#[repr(u16)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug)]
pub enum RejectReason {
    // TODO
}

pub struct SystemEvent {
    pub timestamp: i64,
    pub event_code: EventCode,
}

impl SystemEvent {
    pub const TYPE: u8 = b'S';
}

pub struct OrderAccepted {
    pub timestamp: i64,
    pub user_ref_num: UserRefNum,
    pub side: Side,
    pub quantity: u64,
    pub symbol: Symbol,
    pub price: Price,
    pub time_in_force: TimeInForce,
    pub display: Display,
    pub order_reference_number: u64,
    pub capacity: Capacity,
    pub inter_market_sweep_eligibility: InterMarketSweepEligibility,
    pub cross_type: CrossType,
    pub order_state: OrderState,
    pub cl_ord_id: ClOrdId,
    //pub appendage_length: u16,
    pub optional_appendage: Vector<TagValue>,
}

impl OrderAccepted {
    pub const TYPE: u8 = b'A';
}

pub struct OrderReplaced {
    pub timestamp: i64,
    pub orig_user_ref_num: UserRefNum,
    pub user_ref_num: UserRefNum,
    pub side: Side,
    pub quantity: u64,
    pub symbol: Symbol,
    pub price: Price,
    pub time_in_force: TimeInForce,
    pub display: Display,
    pub order_reference_number: u64,
    pub capacity: Capacity,
    pub inter_market_sweep_eligibility: InterMarketSweepEligibility,
    pub cross_type: CrossType,
    pub order_state: OrderState,
    pub cl_ord_id: ClOrdId,
    //pub appendage_length: u16,
    pub optional_appendage: Vector<TagValue>,
}

impl OrderReplaced {
    pub const TYPE: u8 = b'U';
}

pub struct OrderCanceled {
    pub timestamp: i64,
    pub user_ref_num: UserRefNum,
    pub side: Side,
    pub reason: OrderCancelReason,
    //pub appendage_length: u16,
    pub optional_appendage: Vector<TagValue>,
}

impl OrderCanceled {
    pub const TYPE: u8 = b'C';
}

pub struct AiqCanceled {
    pub timestamp: i64,
    pub user_ref_num: UserRefNum,
    pub decrement_shares: u32, // TODO
    pub quantity_prevent_from_trading: u32,
    pub execution_price: Price,
    pub aiq_strategy: todo!(), // TODO
    //pub appendage_length: u16,
    pub optional_appendage: Vector<TagValue>,
}

impl AiqCanceled {
    pub const TYPE: u8 = b'D';
}

pub struct OrderExecuted {
    pub timestamp: i64,
    pub user_ref_num: UserRefNum,
    pub quantity: u64,
    pub price: Price,
    pub liquidity_flag: LiquidityFlag,
    pub match_number: u64,
    //pub appendage_length: u16,
    pub optional_appendage: Vector<TagValue>,
}

impl OrderExecuted {
    pub const TYPE: u8 = b'E';
}

pub struct BrokenTrade {
    pub timestamp: i64,
    pub user_ref_num: UserRefNum,
    pub match_number: u64,
    pub reason: BrokenReason,
    pub cl_ord_id: ClOrdId,
    //pub appendage_length: u16,
    pub optional_appendage: Vector<TagValue>,
}

impl BrokenTrade {
    pub const TYPE: u8 = b'B';
} 

pub struct Rejected {
    pub timestamp: i64,
    pub user_ref_num: UserRefNum,
    pub reason: RejectReason,
    pub cl_ord_id: ClOrdId,
    //pub appendage_length: u16,
    pub optional_appendage: Vector<TagValue>,
}

impl Rejected {
    pub const TYPE: u8 = b'J';
}

pub struct CancelPending {
    pub timestamp: i64,
    pub user_ref_num: UserRefNum,
    //pub appendage_length: u16,
    pub optional_appendage: Vector<TagValue>,
}

impl CancelPending {
    pub const TYPE: u8 = b'P';
}

pub struct CancelReject {
    pub user_ref_num: UserRefNum,
    //pub appendage_length: u16,
    pub optional_appendage: Vector<TagValue>,
}

impl CancelReject {
    pub const TYPE: u8 = b'I';
}

pub struct OrderPriorityUpdate {
    pub timestamp: i64,
    pub user_ref_num: UserRefNum,
    pub price: Price,
    pub display: Display,
    pub order_reference_number: u64,
    //pub appendage_length: u16,
    pub optional_appendage: Vector<TagValue>,
}

impl OrderPriorityUpdate {
    pub const TYPE: u8 = b'T';
}

pub struct OrderModified {
    pub timestamp: i64,
    pub user_ref_num: UserRefNum,
    pub side: Side,
    pub quantity: u32,
    //pub appendage_length: u16,
    pub optional_appendage: Vector<TagValue>,
}

impl OrderModified {
    pub const TYPE: u8 = b'M';
}

#[repr(u8)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug)]
pub enum RestateReason {
    RefreshOfDisplay = b'R',
    UpdateOfDisplayedPrice = b'P',
}

pub struct OrderRestated {
    pub timestamp: i64,
    pub user_ref_num: UserRefNum,
    pub reason: RestateReason,
    //pub appendage_length: u16,
    pub optional_appendage: Vector<TagValue>,
}

impl OrderRestated {
    pub const TYPE: u8 = b'R';
}

pub struct MassCancelResponse {
    pub timestamp: i64,
    pub user_ref_num: UserRefNum,
    pub firm: Firm,
    pub symbol: Symbol,
    //pub appendage_length: u16,
    pub optional_appendage: Vector<TagValue>,
}

impl MassCancelResponse {
    pub const TYPE: u8 = b'X';
}

pub struct DisableOrderEntryResponse {
    pub timestamp: i64,
    pub user_ref_num: UserRefNum,
    pub firm: Firm,
    //pub appendage_length: u16,
    pub optional_appendage: Vector<TagValue>,
}

impl DisableOrderEntryResponse {
    pub const TYPE: u8 = b'G';
}

pub struct EnableOrderEntryResponse {
    pub timestamp: i64,
    pub user_ref_num: UserRefNum,
    pub firm: Firm,
    //pub appendage_length: u16,
    pub optional_appendage: Vector<TagValue>,
}

impl EnableOrderEntryResponse {
    pub const TYPE: u8 = b'K';
}

pub struct AccountQueryResponse {
    pub timestamp: i64,
    pub next_user_ref_num: UserRefNum,
    //pub appendage_length: u16,
    pub optional_appendage: Vector<TagValue>,
}

impl AccountQueryResponse {
    pub const TYPE: u8 = b'Q';
}
