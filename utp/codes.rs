#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum {
  // CboeBYX represents Cboe BYX Exchange, Inc.
  CboeBYX = b'Y'
  // CboeBZX represents Cboe BZX Exchange, Inc.
  CboeBZX = b'Z'
  // CboeEDGA represents Cboe EDGA Exchange, Inc.
  CboeEDGA = b'J'
  // CboeEDGX represents Cbox EDGX Exchange, Inc.
  CboeEDGX = b'K'
  // Cboe represents Cboe Exchange, Inc.
  Cboe = b'W'

  // NasdaqBX represents Nasdaq BX, Inc.
  NasdaqBX = b'B'
  // NasdaqPHLX represents Nasdaq PHLX LLC.
  NasdaqPHLX = b'X'
  // Nasdaq represents Nasdaq, Inc.
  Nasdaq = b'Q'
  // NasdaqISE represents Nasdaq ISE, LLC.
  NasdaqISE = b'I'

  // Nyse represents New York Stock Exchange LLC.
  Nyse = b'N'
  // NyseArca represents NYSE Arca, Inc.
  NyseArca = b'P'
  // NyseAmerican represents NYSE American, LLC.
  NyseAmerican = b'A'
  // NyseNational represents NYSE National, Inc.
  NyseNational = b'C'
  // NyseChicago represents NYSE Chicago, Inc.
  NyseChicago = b'M'

  // Finra represents Financial Industry Regulatory Authority (FINRA).
  Finra = b'D'
  // Iex represents Investor's Exchange LLC (IEX).
  Iex = b'V'
  // Ltse represents Long-Term Stock Exchange (LTSE).
  Ltse = b'L'
  // Miax represents MIAX Pearl, LLC (MIAX).
  Miax = b'H'
  // Memx represents MEMX LLC (MEMX).
  Memx = b'U'

  // McoiMarketIndependent represents Market Independent (generated by SIP).
  MarketIndependent = b'E'
}
