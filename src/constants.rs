// URLS
pub const WSS_URL: &str = "wss://api.ndax.io/WSGateway";
pub const REST_URL: &str = "https://api.ndax.io:8443/AP/";

// Exchange Data Endpoints
pub const ASSETS: &str = "Assets";

// Order book messages
pub const SUBSCRIBE: &str = "SubscribeLevel2";
pub const UPDATE: &str = "Level2UpdateEvent";
pub const SUBSCRIBE_TRADES: &str = "SubscribeTrades";
pub const UPDATE_TRADES: &str = "TradeDataUpdateEvent";

// REST API Private Endpoints
pub const GET_OPEN_ORDERS_PATH: &str = "GetOpenOrders";
pub const USER_ACCOUNT_INFOS_PATH_URL: &str = "GetUserAccountInfos";
pub const AUTHENTICATE_USER_PATH_URL: &str = "AuthenticateUser";
pub const CANCEL_ALL_ORDERS_PATH_URL: &str = "CancelAllOrders";
