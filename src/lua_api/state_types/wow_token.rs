//! WoW Token sim-state types.

/// One auctionable WoW Token row mirrored back by the AH's WoW Token
/// listings query. Drives the data fed to listeners of
/// `TOKEN_MARKET_PRICE_UPDATED` and used by the WoW Token panel.
#[derive(Debug, Clone)]
pub struct TokenAuctionInfo {
    /// Server-assigned auction id for the listed token.
    pub auction_id: i64,
    /// Listed price in copper.
    pub price: i64,
    /// Owner display name. Empty for tokens listed by the bot/system.
    pub owner: String,
}

/// `C_WowTokenPublic` backing state. Drives commerce eligibility,
/// market price, the listed-token list, and the player's owned-token
/// count surfaced via `UpdateTokenCount`. All fields default to the
/// minimal "commerce on, nothing listed yet" shape so tests can mutate
/// only the fields they care about.
#[derive(Debug, Clone)]
pub struct WowTokenState {
    /// Whether the token shop / commerce system is online. Drives the
    /// first return of `GetCommerceSystemStatus`.
    pub commerce_enabled: bool,
    /// Polling cadence the panel should use, in seconds. Drives the
    /// second return of `GetCommerceSystemStatus`.
    pub poll_seconds: i32,
    /// Whether the player is eligible to receive in-game balance
    /// proceeds from selling a token. Drives the third return of
    /// `GetCommerceSystemStatus`.
    pub balance_enabled: bool,
    /// Most-recent guaranteed-payout market price in copper. Drives
    /// both returns of `GetCurrentMarketPrice` (the second return is
    /// the same as the first per retail) and the value of
    /// `GetGuaranteedPrice`.
    pub current_market_price: i64,
    /// Locked-in payout the seller is guaranteed regardless of market
    /// movement. Drives `GetGuaranteedPrice`.
    pub guaranteed_price: i64,
    /// Currently-listed auctionable tokens. Tests seed this so the
    /// panel can render rows.
    pub listed_auctionable: Vec<TokenAuctionInfo>,
    /// Number of tokens the player currently owns. Refreshed by
    /// `UpdateTokenCount`.
    pub owned_token_count: i32,
    /// Forever server-set gate for the Auction House token category. PTR uses
    /// its native always-enabled override instead of this status input.
    pub auction_house_feature_enabled: bool,
}

impl Default for WowTokenState {
    fn default() -> Self {
        Self {
            commerce_enabled: true,
            poll_seconds: 60,
            balance_enabled: false,
            current_market_price: 0,
            guaranteed_price: 0,
            listed_auctionable: Vec::new(),
            owned_token_count: 0,
            auction_house_feature_enabled: false,
        }
    }
}
