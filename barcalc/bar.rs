pub struct Trade;

pub struct Bar {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: u64,
    number: u64,
    vwap: f64,
    timestamp: i64,
}

impl Bar {
    pub fn update(&mut self, trade: Trade) {
        //
    }
}

macro_rules! build_cond_enum {
    ($($var:ident = $val:literal),+ $(,)?) => {
        #[repr(u8)]
        pub enum Condition {
            $(
                $var = $val,
            )+
        }

        impl Condition {
            pub const fn from_u8(u: u8) -> Option<Self> {
                match u {
                    $(
                        $val => Some(Condition::$var),
                    )+
                    _ => None,
                }
            }
        }
    }
}

build_cond_enum!(
    RegularSale = b'@',
    Acquisition = b'A',
    BunchedTrade = b'B',
    CashSale = b'C',
    Distribution = b'D',
    Placeholder = b'E',
    IntermarketSweep = b'F',
    BunchedSoldTrade = b'G',
    PriceVariationTrade = b'H',
    OddLotTrade = b'I',
    Rule155Trade = b'K',
    SoldLast = b'L',
    MarketCenterOfficialClose = b'M',
    NextDay = b'N',
    OpeningPrints = b'O',
    PriorReferencePrice = b'P',
    MarketCenterOfficialOpen = b'Q',
    Seller = b'R',
    SplitTrade = b'S',
    FormT = b'T',
    ExtendedTradingHours = b'U',
    ContingentTrade = b'V',
    AveragePriceTrade = b'W',
    CrossPeriodicAuctionTrade = b'X',
    YellowFlagRegularTrade = b'Y',
    Sold = b'Z',
    StoppedStock = b'1',
    DerivativelyPriced = b'4',
    ReOpeningPrints = b'5',
    ClosingPrints = b'6',
    QualifiedContingentTrade = b'7',
    PlaceholderFor611Exempt = b'8',
    CorrectedConsolidatedClose = b'9',
);

impl Condition {
    /// Consolidated Processing Guidelines
    pub fn updates_high_low_cpg(self) -> bool {
        use self::Condition::*;
        match self {
            RegularSale
            | Acquisition
            | BunchedTrade
            | Distribution
            | IntermarketSweep
            | BunchedSoldTrade
            | Rule155Trade
            | SoldLast
            | OpeningPrints
            | PriorReferencePrice
            | SplitTrade
            | CrossPeriodicAuctionTrade
            | YellowFlagRegularTrade
            | Sold
            | StoppedStock
            | DerivativelyPriced
            | ReOpeningPrints
            | ClosingPrints
            | CorrectedConsolidatedClose => true,
            _ => false,
        }
    }

    pub fn updates_last_cpg(self) -> bool {
        use self::Condition::*;
        match self {
            RegularSale
            | Acquisition
            | BunchedTrade
            | Distribution
            | IntermarketSweep
            // TODO: Check note
            //| BunchedSoldTrade
            | Rule155Trade
            // TODO: Check note
            | SoldLast
            | OpeningPrints
            // TODO: Check note
            //| PriorReferencePrice
            | SplitTrade
            | CrossPeriodicAuctionTrade
            | YellowFlagRegularTrade
            // TODO: Check note
            //| Sold
            | StoppedStock
            // TODO: Check note
            //| DerivativelyPriced
            | ReOpeningPrints
            | ClosingPrints
            | CorrectedConsolidatedClose => true,
            _ => false,
        }
    }

    /// Market Center Processing Guidelines
    pub fn updates_high_low_mcpg(self) -> bool {
        use self::Condition::*;
        match self {
            RegularSale
            | Acquisition
            | BunchedTrade
            | Distribution
            | IntermarketSweep
            | BunchedSoldTrade
            | Rule155Trade
            | SoldLast
            | MarketCenterOfficialClose
            | OpeningPrints
            | PriorReferencePrice
            | MarketCenterOfficialOpen
            | SplitTrade
            | CrossPeriodicAuctionTrade
            | YellowFlagRegularTrade
            | Sold
            | StoppedStock
            | DerivativelyPriced
            | ReOpeningPrints
            | ClosingPrints => true,
            _ => false,
        }
    }

    pub fn updates_last_mcpg(self) -> bool {
        use self::Condition::*;
        match self {
            RegularSale
            | Acquisition
            | BunchedTrade
            | Distribution
            | IntermarketSweep
            // TODO: Check note
            //| BunchedSoldTrade
            | Rule155Trade
            | SoldLast
            | MarketCenterOfficialClose
            | OpeningPrints
            // TODO: Check note
            //| PriorReferencePrice
            | SplitTrade
            | CrossPeriodicAuctionTrade
            | YellowFlagRegularTrade
            // TODO: Check note
            //| Sold
            | StoppedStock
            // TODO: Check note
            //| DerivativelyPriced
            | ReOpeningPrints
            | ClosingPrints
            | CorrectedConsolidatedClose => true,
            _ => false,
        }
    }

    pub fn updates_volume_mcpg(self) -> bool {
        use self::Condition::*;
        match self {
            RegularSale
            | Acquisition
            | BunchedTrade
            | CashSale
            | Distribution
            | IntermarketSweep
            | BunchedSoldTrade
            | PriceVariationTrade
            | OddLotTrade
            | Rule155Trade
            | SoldLast
            | MarketCenterOfficialClose
            | NextDay
            | OpeningPrints
            | PriorReferencePrice
            | Seller
            | SplitTrade
            | FormT
            | ExtendedTradingHours
            | ContingentTrade
            | AveragePriceTrade
            | CrossPeriodicAuctionTrade
            | YellowFlagRegularTrade
            | Sold
            | StoppedStock
            | DerivativelyPriced
            | ReOpeningPrints
            | ClosingPrints
            | QualifiedContingentTrade => true,
            _ => false,
        }
    }
}
