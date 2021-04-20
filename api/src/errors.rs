use rocket_contrib::json::JsonValue;


/// Struct to hold any json-like error message.
pub struct ErrorMessage(pub Option<JsonValue>);

