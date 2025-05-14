// TODO: encoding/decoding

// TODO: what combo of letters/spaces is valid/invalid? right-padded or left-padded for
// Firm/Symbol/ect.
use std::cmd::{PartialEq, PartialOrd};
use std::fmt;
use std::ops::{Deref, DerefMut};

pub const REVISION: u8 = 4;

pub trait Message {
    const TYPE: u8;
    fn encode(&self) -> Vec<u8>;
    /// Returns the number of bytes encoded into the buf. If the buf is too small, None is
    /// returned.
    fn encode_into(&self, buf: &mut [u8]) -> Option<usize> {
        let encoded = self.encode();
        if encoded.len() > buf.len() {
            return None;
        }
        buf[..encoded.len()].copy_from_slice(&encoded);
        encoded.len()
    }
    /// Write encoded to a writer.
    fn encode_to<W: Write>(&self, w: W) -> io::Result<()> {
        w.write(&self.encode())
    }
}

/* Inbound messages */

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UserRefNum(pub u32);

impl UserRefNum {
    pub const fn incr(self) -> Self {
        Self(self.0 + 1)
    }

    pub const fn add(self, n: u32) -> Self {
        Self(self.0 + n)
    }
}

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

    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
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

impl TimeInForce {
    pub fn to_u8(self) -> u8 {
        self as _
    }
}

#[repr(u8)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug)]
pub enum Display {
    Visible = b'Y',
    Hidden = b'N',
    Attributable = b'A',
}

impl Display {
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

#[repr(u8)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug)]
pub enum Capacity {
    Agency = b'A',
    Principal = b'P',
    Riskless = b'R',
    Other = b'O',
}

impl Capacity {
    pub fn to_u8(self) -> u8 {
        self as _
    }
}

#[repr(u8)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug)]
pub enum InterMarketSweepEligibility {
    Eligible = b'Y',
    NotEligible = b'N',
}
pub type IMSE = InterMarketSweepEligibility;

impl InterMarketSweepEligibility {
    pub fn to_u8(self) -> u8 {
        self as _
    }
}

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

impl CrossType {
    pub fn to_u8(self) -> u8 {
        self as _
    }
}

#[repr(u8)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug, Default)]
pub enum CustomerType {
    RetailDesignatedOrder = b'R',
    #[default]
    NotRetailDesignated = b'N',
    UsePortDefault = b' ',
}

impl CustomerType {
    pub const fn default_enter_order() -> Self {
        CustomerType::UsePortDefault
    }

    pub fn to_u8(self) -> u8 {
        self as _
    }
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

impl PriceType {
    pub const fn default_enter_order() -> Self {
        PriceType::Limit
    }

    pub fn to_u8(self) -> u8 {
        self as _
    }
}

#[repr(u8)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PostOnly {
    PostOnly = b'P',
    #[default]
    No = b'N',
}

impl PostOnly {
    pub const fn default_enter_order() -> Self {
        PostOnly::No
    }

    pub fn to_u8(self) -> u8 {
        self as _
    }
}

#[repr(u8)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TradeNow {
    UsePortDefault = b' ',
    Yes = b'Y',
    #[default]
    No = b'N',
}

impl TradeNow {
    pub const fn from_bool(b: bool) -> Self {
        if b { TradeNow::Yes } else { TradeNow::No }
    }

    pub const fn from_bool_opt(o: Option<bool>) -> Self {
        match o {
            Some(b) => Self::from_bool(b),
            None => TradeNow::UsePortDefault,
        }
    }

    pub const fn default_enter_order() -> Self {
        TradeNow::UsePortDefault
    }

    pub const fn to_bool_opt(self) -> Option<bool> {
        match self {
            TradeNow::UsePortDefault => None,
            TradeNow::Yes => Some(true),
            TradeNow::No => Some(false),
        }
    }

    pub fn to_u8(self) -> u8 {
        self as _
    }
}

impl From<bool> for TradeNow {
    fn from(b: bool) -> Self {
        Self::from_bool(b)
    }
}

impl From<Option<bool>> for TradeNow {
    fn from(o: Option<bool>) -> Self {
        Self::from_bool_opt(o)
    }
}

impl From<TradeNow> for Option<bool> {
    fn from(t: TradeNow) -> Self {
        t.to_bool_opt()
    }
}

#[repr(u8)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SharesLocated {
    Yes = b'Y',
    #[default]
    No = b'N',
}

impl SharesLocated {
    pub const fn from_bool(b: bool) -> Self {
        if b { SharesLocated::Yes } else { SharesLocated::No }
    }

    pub const fn default_enter_order() -> Self {
        SharesLocated::No
    }

    pub const fn to_bool(self) -> bool {
        match self {
            SharesLocated::Yes => true,
            SharesLocated::No => false,
        }
    }

    pub fn to_u8(self) -> u8 {
        self as _
    }
}

impl From<bool> for SharesLocated {
    fn from(b: bool) -> Self {
        Self::from_bool(b)
    }
}

impl From<SharesLocated> for bool {
    fn from(s: SharesLocated) -> Self {
        s.to_bool()
    }
}

#[repr(u8)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug)]
/// Handle instructions.
pub enum HandleInst {
    No = b' ',
    ImbalanceOnly = b'I',
    RetailOrderType1 = b'O',
    RetailOrderType2 = b'T',
    RetailPriceImprovement = b'Q',
    ExtendedLifeContinuous = b'B',
    DirectListingCapitalRaise = b'D',
    /// Retail Price Improvement, Hidden Price Improvement Indicator.
    RPIHPII = b'R',
}

impl HandleInst {
    pub const fn default_enter_order() -> Self {
        HandleInst::No
    }

    pub fn to_u8(self) -> u8 {
        self as _
    }
}

#[repr(u8)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug, Default)]
pub enum BboWeightIndicator {
    #[default]
    Unspecified = b' ',
    Pct0 = b'0',
    Pct1 = b'1',
    Pct2 = b'2',
    Pct3 = b'3',
    SetQbbo = b'S',
    ImproveNbbo = b'N',
}

impl BboWeightIndicator {
    pub fn to_u8(self) -> u8 {
        self as _
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BrokerCode([u8; 4]);

impl BrokerCode {
    // FIXME: is comment below correct?
    // A valid new broker is one that is 4 bytes long and contains all caps or spaces.
    pub fn new<B: AsRef<[u8]>(bar: B) -> Result<Self, B> {
        let bytes = bar.as_ref();
        if bytes.len() != 4 {
            return Err(bar);
        }
        let mut in_letters = false;
        for i in 0..4 {
            match bytes[i] {
                b'A'..=b'Z' => in_letters = true,
                b' ' => {
                    if in_letters {
                        return Err(bar);
                    }
                },
                _ => return Err(bar),
            }
        }
        Ok(Self(bytes.try_into().unwrap()))
    }

    pub const fn empty() -> Self {
        Self([b' '; 4])
    }

    pub fn as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.0).ok()
    }
}

/*
impl Default for BrokerCode {
    fn default() -> Self {
        Self::empty()
    }
}
*/

impl fmt::Display for BrokerCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!("{}", self.as_str().unwrap_or("????"))
    }
}

#[repr(u8)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug)]
pub enum OptionValue {
    // FIXME: numeric
    SecondaryOrdRefNum(u64) = 1,
    Firm(Firm) = 2,
    MinQty(u32) = 3,
    CustomerType(CustomerType) = 4,
    MaxFloor(u32) = 5,
    PriceType(PriceType) = 6,
    PegOffset(nasdaq::SignedPrice) = 7,
    DiscretionPrice(nasdaq::Price) = 9,
    // TODO: verify valid price type
    DiscretionPriceType(PriceType) = 10,
    DiscretionPegOffset(nasdaq::SignedPrice) = 11,
    PostOnly(PostOnly) = 12,
    RandomReserves(u32) = 13,
    #[deprecated]
    Route(u32) = 14,
    // TODO: validation
    ExpireTime(u32) = 15,
    TradeNow(TradeNow) = 16,
    HandleInst(HandleInst) = 17,
    BboWeightIndicator(BboWeightIndicator) = 18,
    DisplayQuantity(u32) = 22,
    DisplayPrice(nasdaq::Price) = 23,
    GroupId(u16) = 24,
    SharesLocated(SharesLocated) = 25,
    LocateBroker(Broker) = 26,
    Side(Side) = 27,
    UserRefIdx(u8) = 28,
}

impl OptionValue {
    pub fn size(self) -> usize {
        match self {
            OptionTag::SecondaryOrdRefNum(_) => 8,
            OptionTag::Firm(_) => 4,
            OptionTag::MinQty(_) => 4,
            OptionTag::CustomerType(_) => 1,
            OptionTag::MaxFloor(_) => 4,
            OptionTag::PriceType(_) => 1,
            OptionTag::PegOffset(_) => 4,
            OptionTag::DiscretionPrice(_) => 8,
            OptionTag::DiscretionPriceType(_) => 1,
            OptionTag::DiscretionPegOffset(_) => 4,
            OptionTag::PostOnly(_) => 1,
            OptionTag::RandomReserves(_) => 4,
            OptionTag::Route(_) => 4,
            OptionTag::ExpireTime(_) => 4,
            OptionTag::TradeNow(_) => 1,
            OptionTag::HandleInst(_) => 1,
            OptionTag::BboWeightIndicator(_) => 1,
            OptionTag::DisplayQuantity(_) => 4,
            OptionTag::DisplayPrice(_) => 8,
            OptionTag::GroupId(_) => 2,
            OptionTag::SharesLocated(_) => 1,
            OptionTag::LocateBroker(_) => 4,
            OptionTag::Side(_) => 1,
            OptionTag::UserRefIdx(_) => 1,
        }
    }

    fn append_to(self, buf: &mut Vec<u8>) {
        match self {
            OptionTag::SecondaryOrdRefNum(v) => buf.extend_from_slice(&v.to_be_bytes()),
            OptionTag::Firm(v) => buf.extend_from_slice(&v.as_bytes()),
            OptionTag::MinQty(v) => buf.extend_from_slice(&v.to_be_bytes()),
            OptionTag::CustomerType(v) => buf.push(v.to_u8()),
            OptionTag::MaxFloor(v) => buf.extend_from_slice(&v.to_be_bytes()),
            OptionTag::PriceType(v) => buf.push(v.to_u8()),
            OptionTag::PegOffset(v) => buf.extend_from_slice(&v.to_bytes()),
            OptionTag::DiscretionPrice(v) => buf.extend_from_slice(&v.to_bytes()),
            OptionTag::DiscretionPriceType(v) => buf.push(v.to_u8()),
            OptionTag::DiscretionPegOffset(v) => buf.extend_from_slice(&v.to_bytes()),
            OptionTag::PostOnly(v) => buf.push(v.to_u8()),
            OptionTag::RandomReserves(v) => buf.extend_from_slice(&v.to_be_bytes()),
            OptionTag::Route(v) => buf.extend_from_slice(&v.to_be_bytes()),
            OptionTag::ExpireTime(v) => buf.extend_from_slice(&v.to_be_bytes()),
            OptionTag::TradeNow(v) => buf.push(v.to_u8()),
            OptionTag::HandleInst(v) => buf.push(v.to_u8()),
            OptionTag::BboWeightIndicator(v) => buf.push(v.to_u8()),
            OptionTag::DisplayQuantity(v) => buf.extend_from_slice(&v.to_be_bytes()),
            OptionTag::DisplayPrice(v) => buf.extend_from_slice(&v.to_bytes()),
            OptionTag::GroupId(v) => buf.extend_from_slice(&v.to_be_bytes()),
            OptionTag::SharesLocated(v) => buf.push(v.to_u8()),
            OptionTag::LocateBroker(v) => buf.extend_from_slice(&v.as_bytes()),
            OptionTag::Side(v) => buf.push(v.to_u8()),
            OptionTag::UserRefIdx(v) => buf.push(v),
        }
    }
}

pub struct TagValue {
    pub option_value: OptionValue,
}

impl TagValue {
    pub fn length(self) -> u8 {
        self.option_value.size() as _
    }
}

#[repr(transparent)]
pub struct OptionalAppendage(Vec<TagValue>);

impl OptionalAppendage {
    pub const MAX_LEN: usize = u16::MAX as usize;

    pub fn new(v: impl Into<Vec<TagValue>>) -> Result<Self, Vec<TagValue>> {
        let v = v.into();
        if v.len() > Self::MAX_LEN {
            return Err(v);
        }
        Ok(Self(v))
    }

    pub fn push(&mut self, tv: TagValue) -> Result<(), TagValue> {
        if self.len() >= Self::MAX_LEN {
            return Err(tv);
        }
        self.0.push(tv);
        Ok(())
    }

    pub fn insert(&mut self, index: usize, tv: TagValue) -> Result<(), TagValue> {
        if self.len() >= Self::MAX_LEN {
            return Err(tv);
        }
        self.0.insert(index, tv);
        Ok(())
    }

    pub fn pop(&mut self) -> Option<TagValue> {
        self.0.pop()
    }

    pub fn remove(&mut self, index: usize) -> TagValue {
        self.0.remove(index);
    }

    pub fn into_iter(self) -> impl Iterator<Item=TagValue> {
        self.0.into_iter()
    }

    pub fn into_inner(self) -> Vec<TagValue> {
        self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    fn append_to(&self, buf: &mut Vec<u8>, encode_len: bool) {
        let l = self.len().max(Self::MAX_LEN);
        if encode_len {
            buf.extend_from_slice(l.to_be_bytes());
        }
        for i in 0..l {
            self[i].append_to(buf);
        }
    }
}

impl Deref for OptionalAppendage {
    type Target = [TagValue];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for OptionalAppendage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Firm([u8; 4]);

impl Firm {
    /// A valid new firm is one that is 4 bytes long and contains all caps or spaces.
    pub fn new<B: AsRef<[u8]>(bar: B) -> Result<Self, B> {
        let bytes = bar.as_ref();
        if bytes.len() != 4 {
            return Err(bar);
        }
        let mut in_letters = false;
        for i in 0..4 {
            match bytes[i] {
                b'A'..=b'Z' => in_letters = true,
                b' ' => {
                    if in_letters {
                        return Err(bar);
                    }
                },
                _ => return Err(bar),
            }
        }
        Ok(Self(bytes.try_into().unwrap()))
    }

    pub const fn empty() -> Self {
        Self([b' '; 4])
    }

    pub fn as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.0).ok()
    }
}

impl Default for Firm {
    fn default() -> Self {
        Self::empty()
    }
}

impl fmt::Display for Firm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!("{}", self.as_str().unwrap_or("????"))
    }
}

pub struct EnterOrder {
    pub user_ref_num: UserRefNum,
    pub side: OrderSide,
    pub quantity: u32,
    pub symbol: nasdaq::Symbol,
    pub price: nasdaq::Price,
    pub time_in_force: TimeInForce,
    pub display: Display,
    pub capacity: Capacity,
    pub inter_market_sweep_eligibility: InterMarketSweepEligibility,
    pub cross_type: CrossType,
    pub cl_ord_id: ClOrdId,
    pub optional_appendage: OptionalAppendage,
}

impl Message for EnterOrder {
    pub const TYPE: u8 = b'O';

    fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(47);
        buf.push(Self::TYPE);
        buf.extend_from_slice(&self.user_ref_num.to_bytes());
        buf.push(self.side.to_u8());
        buf.extend_from_slice(&self.quantity.to_be_bytes());
        buf.extend_from_slice(self.symbol.as_bytes());
        buf.extend_from_slice(&self.price.to_bytes());
        buf.push(self.time_in_force.to_u8());
        buf.push(self.display.to_u8());
        buf.push(self.capacity.to_u8());
        buf.push(self.inter_market_sweep_eligibility.to_u8());
        buf.push(self.cross_type.to_u8());
        buf.extend_from_slice(self.cl_ord_id.as_bytes());
        self.optional_appendage.append_to(&mut buf, true);
    }
}

pub struct ReplaceOrderRequest {
    pub orig_user_ref_num: UserRefNum,
    pub user_ref_num: UserRefNum,
    pub quantity: u32,
    pub price: nasdaq::Price,
    pub time_in_force: TimeInForce,
    pub display: Display,
    pub inter_market_sweep_eligibility: InterMarketSweepEligibility,
    pub cl_ord_id: ClOrdId,
    pub optional_appendage: OptionalAppendage,
}

impl ReplaceOrderRequest {
    pub const TYPE: u8 = b'U';
}

pub struct CancelOrderRequest {
    pub user_ref_num: UserRefNum,
    pub quantity: u32,
    pub optional_appendage: OptionalAppendage,
}

impl CancelOrderRequest {
    pub const TYPE: u8 = b'X';
}

pub struct ModifyOrderRequest {
    pub user_ref_num: UserRefNum,
    pub side: Side,
    pub quantity: u32,
    pub optional_appendage: OptionalAppendage,
}

impl ModifyOrderRequest {
    pub const TYPE: u8 = b'M';
}

pub struct MassCancerRequest {
    pub user_ref_num: UserRefNum,
    pub firm: Firm,
    pub symbol: nasdaq::Symbol,
    pub optional_appendage: OptionalAppendage,
}

impl MassOrderRequest {
    pub const TYPE: u8 = b'C';
}

pub struct DisableOrderEntryRequest {
    pub user_ref_num: UserRefNum,
    pub firm: Firm,
    pub optional_appendage: OptionalAppendage,
}

impl DisableOrderEntryRequest {
    pub const TYPE: u8 = b'D';
}

pub struct EnableOrderEntryRequest {
    pub user_ref_num: UserRefNum,
    pub firm: Firm,
    pub optional_appendage: OptionalAppendage,
}

impl EnableOrderEntryRequest {
    pub const TYPE: u8 = b'E';
}

pub struct AccountQueryRequest {
    pub optional_appendage: OptionalAppendage,
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

impl EventCode {
    pub fn to_u8(self) -> u8 {
        self as _
    }
}

#[repr(u8)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug)]
pub enum OrderState {
    Live = b'L',
    Dead = b'D',
}

impl OrderState {
    pub fn to_u8(self) -> u8 {
        self as _
    }
}

#[repr(u8)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug)]
pub enum OrderCancelReason {
    /// This order cannot be executed because of a regulatory restriction (e.g.: trade through
    /// restrictions).
    RegulatoryRestriction = b'D',
    /// Closed. Any DAY order that was received after the closing cross is complete in a given
    /// symbol will receive this cancel reason.
    Closed = b'E',
    /// Post Only Cancel. This Post Only order was cancelled because it would have been price slid
    /// for NMS.
    PostOnlyNms = b'F',
    /// Post Only Cancel. This Post Only order was cancelled because it would have been price slid
    /// due to a contra side displayed order on the book.
    PostOnlyContra = b'G',
    /// Halted. The on-open order was canceled because the symbol remained halted after the opening
    /// cross completed.
    Halted = b'H',
    /// Immediate or Cancel Order.
    ImmediateOrCancel = b'I',
    /// This order cannot be executed because of Market Collars.
    MarketCollars = b'K',
    /// Self Match Prevention. This order was cancelled because it would have executed with an
    /// existing order entered by the same MPID.
    SelfMatchPrevention = b'Q',
    /// Supervisory. The order was manually canceled or reduced by an NASDAQ supervisory terminal.
    Supervisory = b'S',
    /// Timeout. The Time In Force for this order has expired.
    Timeout = b'T',
    /// User requested cancel. Sent in response to a Cancel Request Message.
    UserRequested = b'U',
    /// Open Protection. Orders that are cancelled as a result of the Opening Price Protection
    /// Threshold.
    OpenProtection = b'X',
    /// System cancel. This order was canceled by the system.
    SystemCancel = b'Z',
    /// Company Direct Listing Capital Raise order exceeds allowable shares offered.
    Exceeds = b'e',
}

impl OrderCancelReason {
    pub fn to_u8(self) -> u8 {
        self as _
    }
}

#[repr(u8)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug)]
pub enum LiquidityFlag {
    Added = b'A',
    ClosingCross = b'C',
    /// Retail designated execution that added displayed liquidity.
    RetailDesigExec = b'e',
    HaltIpoCross = b'H',
    AfterHoursClosingCross = b'i',
    /// Non-displayed adding liquidity.
    NonDisplayed = b'J',
    /// RPI (Retail Price Improving) order provides liquidity.
    RpiOrder = b'j',
    HaltCross = b'K',
    /// Closing Cross (imbalance-only).
    ClosingCrossImbalanceOnly = b'L',
    /// Opening Cross (imbalance-only).
    OpeningCrossImbalanceOnly = b'M',
    /// Removed liquidity at a midpoint.
    RemovedLiquidity = b'm',
    PassiveMidpointExecution = b'N',
    // Midpoint extended life order execution
    MidpointExtended = b'n',
    OpeningCross = b'O',
    /// Removed price improving non-displayed liquidity
    RemovedPrice = b'p',
    /// RMO retail order removes non-RPI midpoint liquidity
    RmoRetailOrder = b'q',
    Removed = b'R',
    /// Retail Order removes RPI liquidity.
    RetailOrderRpi = b'r',
    /// Retail Order removes price improving non-displayed liquidity other than RPI liquidity.
    RetailOrderPrice = b't',
    /// Added non-displayed liquidity via a Reserve order.
    ReserveOrder = b'u',
    SupplementalOrderExecution = b'0',
    /// Displayed, liquidity-adding order improves the NBBO.
    DisplayedNbbo = b'7',
    /// Displayed, liquidity-adding order sets the QBBO while joining the NBBO.
    DisplayedQbbo = b'7',
    /// RPI order provides liquidity, No RPII.
    RpiOrderNoRpii = b'1',
}

impl LiquidityFlag {
    pub fn to_u8(self) -> u8 {
        self as _
    }
}

#[repr(u8)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug)]
pub enum BrokenReason {
    Erroneuos = b'E',
    Consent = b'C',
    Supervisory = b'S',
    External = b'X',
}

impl BrokenReason {
    pub fn to_u8(self) -> u8 {
        self as _
    }
}

#[repr(u16)]
#[derive(Cloone, Copy, PartialEq, Eq, Debug)]
pub enum RejectReason {
    QuoteUnavailable = 0x0001,
    DestinationClosed = 0x0002,
    InvalidDisplay = 0x0003,
    InvalidMaxFloor = 0x0004,
    InvalidPegType = 0x0005,
    FatFinger = 0x0006,
    Halted = 0x0007,
    IsoNotAllowed = 0x0008,
    InvalidSlide = 0x0009,
    ProcessingError = 0x000A,
    CancelPending = 0x000B,
    FirmNotAuthorized = 0x000C,
    InvalidMinQuantity = 0x000D,
    NoClosingReferencePrice = 0x000E,
    Other = 0x000F,
    CancelNotAllowed = 0x0010,
    PeggingNotAllowed = 0x0011,
    CrossedMarket = 0x0012,
    InvalidQuantity = 0x0013,
    InvalidCrossOrder = 0x0014,
    ReplaceNotAllowed = 0x0015,
    RoutingNotAllowed = 0x0016,
    InvalidSymbol = 0x0017,
    Test = 0x0018,
    LateLocTooAggressive = 0x0019,
    RetailNotAllowed = 0x001A,
    InvalidMidpointPostOnlyPrice = 0x001B,
    InvalidDestination = 0x001C,
    InvalidPrice = 0x001D,
    SharesExceedThreshold = 0x001E,
    ExceedsMaximumAllowedNotionalValue = 0x001F,
    RiskAggregateExposureExceeded = 0x0020,
    RiskMarketImpact = 0x0021,
    RiskRestrictedStock = 0x0022,
    RiskShortSellRestricted = 0x0023,
    RiskIsoNotAllowed = 0x0024,
    RiskExceedsAdvLimit = 0x0025,
    RiskFatFinger = 0x0026,
    RiskLocateRequired = 0x0027,
    RiskSymbolMessageRateRestriction = 0x0028,
    RiskPortMessageRateRestriction = 0x0029,
    RiskDuplicateMessageRateRestriction = 0x002A,
    RiskShortSellNotAllowed = 0x002B,
    RiskMarketOrderNotAllowed = 0x002C,
    RiskPreMarketNotAllowed = 0x002D,
    RiskPostMarketNotAllowed = 0x002E,
    RiskShortSellExemptNotAllowed = 0x002F,
    RiskSingleOrderNotionalExceeded = 0x0030,
    RiskMaxQuantityExceeded = 0x0031,
    RegShoStateNotAvailable = 0x0032,
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
    pub symbol: nasdaq::Symbol,
    pub price: Price,
    pub time_in_force: TimeInForce,
    pub display: Display,
    pub order_reference_number: u64,
    pub capacity: Capacity,
    pub inter_market_sweep_eligibility: InterMarketSweepEligibility,
    pub cross_type: CrossType,
    pub order_state: OrderState,
    pub cl_ord_id: ClOrdId,
    pub optional_appendage: OptionalAppendage,
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
    pub symbol: nasdaq::Symbol,
    pub price: nasdaq::Price,
    pub time_in_force: TimeInForce,
    pub display: Display,
    pub order_reference_number: u64,
    pub capacity: Capacity,
    pub inter_market_sweep_eligibility: InterMarketSweepEligibility,
    pub cross_type: CrossType,
    pub order_state: OrderState,
    pub cl_ord_id: ClOrdId,
    pub optional_appendage: OptionalAppendage,
}

impl OrderReplaced {
    pub const TYPE: u8 = b'U';
}

pub struct OrderCanceled {
    pub timestamp: i64,
    pub user_ref_num: UserRefNum,
    pub side: Side,
    pub reason: OrderCancelReason,
    pub optional_appendage: OptionalAppendage,
}

impl OrderCanceled {
    pub const TYPE: u8 = b'C';
}

pub struct AiqCanceled {
    pub timestamp: i64,
    pub user_ref_num: UserRefNum,
    pub decrement_shares: u32,
    pub quantity_prevent_from_trading: u32,
    pub execution_price: nasdaq::Price,
    pub aiq_strategy: todo!(), // TODO
    pub optional_appendage: OptionalAppendage,
}

impl AiqCanceled {
    pub const TYPE: u8 = b'D';
}

pub struct OrderExecuted {
    pub timestamp: i64,
    pub user_ref_num: UserRefNum,
    pub quantity: u64,
    pub price: nasdaq::Price,
    pub liquidity_flag: LiquidityFlag,
    pub match_number: u64,
    pub optional_appendage: OptionalAppendage,
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
    pub optional_appendage: OptionalAppendage,
}

impl BrokenTrade {
    pub const TYPE: u8 = b'B';
} 

pub struct Rejected {
    pub timestamp: i64,
    pub user_ref_num: UserRefNum,
    pub reason: RejectReason,
    pub cl_ord_id: ClOrdId,
    pub optional_appendage: OptionalAppendage,
}

impl Rejected {
    pub const TYPE: u8 = b'J';
}

pub struct CancelPending {
    pub timestamp: i64,
    pub user_ref_num: UserRefNum,
    pub optional_appendage: OptionalAppendage,
}

impl CancelPending {
    pub const TYPE: u8 = b'P';
}

pub struct CancelReject {
    pub user_ref_num: UserRefNum,
    pub optional_appendage: OptionalAppendage,
}

impl CancelReject {
    pub const TYPE: u8 = b'I';
}

pub struct OrderPriorityUpdate {
    pub timestamp: i64,
    pub user_ref_num: UserRefNum,
    pub price: nasdaq::Price,
    pub display: Display,
    pub order_reference_number: u64,
    pub optional_appendage: OptionalAppendage,
}

impl OrderPriorityUpdate {
    pub const TYPE: u8 = b'T';
}

pub struct OrderModified {
    pub timestamp: i64,
    pub user_ref_num: UserRefNum,
    pub side: Side,
    pub quantity: u32,
    pub optional_appendage: OptionalAppendage,
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

impl RestateReason {
    pub fn to_u8(self) -> u8 {
        self as _
    }
}

pub struct OrderRestated {
    pub timestamp: i64,
    pub user_ref_num: UserRefNum,
    pub reason: RestateReason,
    pub optional_appendage: OptionalAppendage,
}

impl OrderRestated {
    pub const TYPE: u8 = b'R';
}

pub struct MassCancelResponse {
    pub timestamp: i64,
    pub user_ref_num: UserRefNum,
    pub firm: Firm,
    pub symbol: nasdaq::Symbol,
    pub optional_appendage: OptionalAppendage,
}

impl MassCancelResponse {
    pub const TYPE: u8 = b'X';
}

pub struct DisableOrderEntryResponse {
    pub timestamp: i64,
    pub user_ref_num: UserRefNum,
    pub firm: Firm,
    pub optional_appendage: OptionalAppendage,
}

impl DisableOrderEntryResponse {
    pub const TYPE: u8 = b'G';
}

pub struct EnableOrderEntryResponse {
    pub timestamp: i64,
    pub user_ref_num: UserRefNum,
    pub firm: Firm,
    pub optional_appendage: OptionalAppendage,
}

impl EnableOrderEntryResponse {
    pub const TYPE: u8 = b'K';
}

pub struct AccountQueryResponse {
    pub timestamp: i64,
    pub next_user_ref_num: UserRefNum,
    pub optional_appendage: OptionalAppendage,
}

impl AccountQueryResponse {
    pub const TYPE: u8 = b'Q';
}
