// GENERATED FILE - DO NOT EDIT.
// Source: protocol/schema/ via protocol/generated/protocol.bundle.schema.json.
// Regenerate with `npm run generate:rust` in protocol/.

#![allow(clippy::redundant_closure_call)]
#![allow(clippy::needless_lifetimes)]
#![allow(clippy::match_single_binding)]
#![allow(clippy::clone_on_copy)]

#[doc = r" Error types."]
pub mod error {
    #[doc = r" Error from a `TryFrom` or `FromStr` implementation."]
    pub struct ConversionError(::std::borrow::Cow<'static, str>);
    impl ::std::error::Error for ConversionError {}
    impl ::std::fmt::Display for ConversionError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
            ::std::fmt::Display::fmt(&self.0, f)
        }
    }
    impl ::std::fmt::Debug for ConversionError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
            ::std::fmt::Debug::fmt(&self.0, f)
        }
    }
    impl From<&'static str> for ConversionError {
        fn from(value: &'static str) -> Self {
            Self(value.into())
        }
    }
    impl From<String> for ConversionError {
        fn from(value: String) -> Self {
            Self(value.into())
        }
    }
}
#[doc = "payload.timestamp is diagnostic only. Online/offline decisions use the server monotonic clock (doc 11.1)."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"AgentHeartbeatMessage\","]
#[doc = "  \"description\": \"payload.timestamp is diagnostic only. Online/offline decisions use the server monotonic clock (doc 11.1).\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"deviceId\","]
#[doc = "    \"payload\","]
#[doc = "    \"protocolVersion\","]
#[doc = "    \"requestId\","]
#[doc = "    \"sessionId\","]
#[doc = "    \"type\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"deviceId\": {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    },"]
#[doc = "    \"payload\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"timestamp\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"timestamp\": {"]
#[doc = "          \"$ref\": \"#/$defs/DateTime\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"protocolVersion\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"enum\": ["]
#[doc = "        1"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"requestId\": {"]
#[doc = "      \"type\": \"null\""]
#[doc = "    },"]
#[doc = "    \"sessionId\": {"]
#[doc = "      \"type\": \"null\""]
#[doc = "    },"]
#[doc = "    \"type\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"agent.heartbeat\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct AgentHeartbeatMessage {
    #[serde(rename = "deviceId")]
    pub device_id: Uuid,
    pub payload: AgentHeartbeatMessagePayload,
    #[serde(rename = "protocolVersion")]
    pub protocol_version: AgentHeartbeatMessageProtocolVersion,
    #[serde(rename = "requestId")]
    pub request_id: (),
    #[serde(rename = "sessionId")]
    pub session_id: (),
    #[serde(rename = "type")]
    pub type_: AgentHeartbeatMessageType,
}
impl AgentHeartbeatMessage {
    pub fn builder() -> builder::AgentHeartbeatMessage {
        Default::default()
    }
}
#[doc = "`AgentHeartbeatMessagePayload`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"timestamp\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"timestamp\": {"]
#[doc = "      \"$ref\": \"#/$defs/DateTime\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct AgentHeartbeatMessagePayload {
    pub timestamp: DateTime,
}
impl AgentHeartbeatMessagePayload {
    pub fn builder() -> builder::AgentHeartbeatMessagePayload {
        Default::default()
    }
}
#[doc = "`AgentHeartbeatMessageProtocolVersion`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"integer\","]
#[doc = "  \"enum\": ["]
#[doc = "    1"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct AgentHeartbeatMessageProtocolVersion(i64);
impl ::std::ops::Deref for AgentHeartbeatMessageProtocolVersion {
    type Target = i64;
    fn deref(&self) -> &i64 {
        &self.0
    }
}
impl ::std::convert::From<AgentHeartbeatMessageProtocolVersion> for i64 {
    fn from(value: AgentHeartbeatMessageProtocolVersion) -> Self {
        value.0
    }
}
impl ::std::convert::TryFrom<i64> for AgentHeartbeatMessageProtocolVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: i64) -> ::std::result::Result<Self, self::error::ConversionError> {
        if ![1_i64].contains(&value) {
            Err("invalid value".into())
        } else {
            Ok(Self(value))
        }
    }
}
impl<'de> ::serde::Deserialize<'de> for AgentHeartbeatMessageProtocolVersion {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        Self::try_from(<i64>::deserialize(deserializer)?)
            .map_err(|e| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[doc = "`AgentHeartbeatMessageType`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"agent.heartbeat\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum AgentHeartbeatMessageType {
    #[serde(rename = "agent.heartbeat")]
    AgentHeartbeat,
}
impl ::std::fmt::Display for AgentHeartbeatMessageType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::AgentHeartbeat => f.write_str("agent.heartbeat"),
        }
    }
}
impl ::std::str::FromStr for AgentHeartbeatMessageType {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "agent.heartbeat" => Ok(Self::AgentHeartbeat),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for AgentHeartbeatMessageType {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for AgentHeartbeatMessageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for AgentHeartbeatMessageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`AgentHelloAckMessage`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"AgentHelloAckMessage\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"deviceId\","]
#[doc = "    \"payload\","]
#[doc = "    \"protocolVersion\","]
#[doc = "    \"requestId\","]
#[doc = "    \"sessionId\","]
#[doc = "    \"type\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"deviceId\": {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    },"]
#[doc = "    \"payload\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"heartbeatIntervalMs\","]
#[doc = "        \"serverTime\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"heartbeatIntervalMs\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"enum\": ["]
#[doc = "            20000"]
#[doc = "          ]"]
#[doc = "        },"]
#[doc = "        \"serverTime\": {"]
#[doc = "          \"$ref\": \"#/$defs/DateTime\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"protocolVersion\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"enum\": ["]
#[doc = "        1"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"requestId\": {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    },"]
#[doc = "    \"sessionId\": {"]
#[doc = "      \"type\": \"null\""]
#[doc = "    },"]
#[doc = "    \"type\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"agent.helloAck\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct AgentHelloAckMessage {
    #[serde(rename = "deviceId")]
    pub device_id: Uuid,
    pub payload: AgentHelloAckMessagePayload,
    #[serde(rename = "protocolVersion")]
    pub protocol_version: AgentHelloAckMessageProtocolVersion,
    #[serde(rename = "requestId")]
    pub request_id: Uuid,
    #[serde(rename = "sessionId")]
    pub session_id: (),
    #[serde(rename = "type")]
    pub type_: AgentHelloAckMessageType,
}
impl AgentHelloAckMessage {
    pub fn builder() -> builder::AgentHelloAckMessage {
        Default::default()
    }
}
#[doc = "`AgentHelloAckMessagePayload`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"heartbeatIntervalMs\","]
#[doc = "    \"serverTime\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"heartbeatIntervalMs\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"enum\": ["]
#[doc = "        20000"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"serverTime\": {"]
#[doc = "      \"$ref\": \"#/$defs/DateTime\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct AgentHelloAckMessagePayload {
    #[serde(rename = "heartbeatIntervalMs")]
    pub heartbeat_interval_ms: AgentHelloAckMessagePayloadHeartbeatIntervalMs,
    #[serde(rename = "serverTime")]
    pub server_time: DateTime,
}
impl AgentHelloAckMessagePayload {
    pub fn builder() -> builder::AgentHelloAckMessagePayload {
        Default::default()
    }
}
#[doc = "`AgentHelloAckMessagePayloadHeartbeatIntervalMs`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"integer\","]
#[doc = "  \"enum\": ["]
#[doc = "    20000"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct AgentHelloAckMessagePayloadHeartbeatIntervalMs(i64);
impl ::std::ops::Deref for AgentHelloAckMessagePayloadHeartbeatIntervalMs {
    type Target = i64;
    fn deref(&self) -> &i64 {
        &self.0
    }
}
impl ::std::convert::From<AgentHelloAckMessagePayloadHeartbeatIntervalMs> for i64 {
    fn from(value: AgentHelloAckMessagePayloadHeartbeatIntervalMs) -> Self {
        value.0
    }
}
impl ::std::convert::TryFrom<i64> for AgentHelloAckMessagePayloadHeartbeatIntervalMs {
    type Error = self::error::ConversionError;
    fn try_from(value: i64) -> ::std::result::Result<Self, self::error::ConversionError> {
        if ![20000_i64].contains(&value) {
            Err("invalid value".into())
        } else {
            Ok(Self(value))
        }
    }
}
impl<'de> ::serde::Deserialize<'de> for AgentHelloAckMessagePayloadHeartbeatIntervalMs {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        Self::try_from(<i64>::deserialize(deserializer)?)
            .map_err(|e| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[doc = "`AgentHelloAckMessageProtocolVersion`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"integer\","]
#[doc = "  \"enum\": ["]
#[doc = "    1"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct AgentHelloAckMessageProtocolVersion(i64);
impl ::std::ops::Deref for AgentHelloAckMessageProtocolVersion {
    type Target = i64;
    fn deref(&self) -> &i64 {
        &self.0
    }
}
impl ::std::convert::From<AgentHelloAckMessageProtocolVersion> for i64 {
    fn from(value: AgentHelloAckMessageProtocolVersion) -> Self {
        value.0
    }
}
impl ::std::convert::TryFrom<i64> for AgentHelloAckMessageProtocolVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: i64) -> ::std::result::Result<Self, self::error::ConversionError> {
        if ![1_i64].contains(&value) {
            Err("invalid value".into())
        } else {
            Ok(Self(value))
        }
    }
}
impl<'de> ::serde::Deserialize<'de> for AgentHelloAckMessageProtocolVersion {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        Self::try_from(<i64>::deserialize(deserializer)?)
            .map_err(|e| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[doc = "`AgentHelloAckMessageType`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"agent.helloAck\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum AgentHelloAckMessageType {
    #[serde(rename = "agent.helloAck")]
    AgentHelloAck,
}
impl ::std::fmt::Display for AgentHelloAckMessageType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::AgentHelloAck => f.write_str("agent.helloAck"),
        }
    }
}
impl ::std::str::FromStr for AgentHelloAckMessageType {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "agent.helloAck" => Ok(Self::AgentHelloAck),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for AgentHelloAckMessageType {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for AgentHelloAckMessageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for AgentHelloAckMessageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`AgentHelloMessage`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"AgentHelloMessage\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"deviceId\","]
#[doc = "    \"payload\","]
#[doc = "    \"protocolVersion\","]
#[doc = "    \"requestId\","]
#[doc = "    \"sessionId\","]
#[doc = "    \"type\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"deviceId\": {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    },"]
#[doc = "    \"payload\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"agentVersion\","]
#[doc = "        \"capabilities\","]
#[doc = "        \"platform\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"agentVersion\": {"]
#[doc = "          \"$ref\": \"#/$defs/Semver\""]
#[doc = "        },"]
#[doc = "        \"capabilities\": {"]
#[doc = "          \"type\": \"array\","]
#[doc = "          \"items\": {"]
#[doc = "            \"$ref\": \"#/$defs/Capability\""]
#[doc = "          },"]
#[doc = "          \"maxItems\": 2,"]
#[doc = "          \"minItems\": 1,"]
#[doc = "          \"uniqueItems\": true"]
#[doc = "        },"]
#[doc = "        \"platform\": {"]
#[doc = "          \"$ref\": \"#/$defs/Platform\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"protocolVersion\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"enum\": ["]
#[doc = "        1"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"requestId\": {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    },"]
#[doc = "    \"sessionId\": {"]
#[doc = "      \"type\": \"null\""]
#[doc = "    },"]
#[doc = "    \"type\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"agent.hello\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct AgentHelloMessage {
    #[serde(rename = "deviceId")]
    pub device_id: Uuid,
    pub payload: AgentHelloMessagePayload,
    #[serde(rename = "protocolVersion")]
    pub protocol_version: AgentHelloMessageProtocolVersion,
    #[serde(rename = "requestId")]
    pub request_id: Uuid,
    #[serde(rename = "sessionId")]
    pub session_id: (),
    #[serde(rename = "type")]
    pub type_: AgentHelloMessageType,
}
impl AgentHelloMessage {
    pub fn builder() -> builder::AgentHelloMessage {
        Default::default()
    }
}
#[doc = "`AgentHelloMessagePayload`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"agentVersion\","]
#[doc = "    \"capabilities\","]
#[doc = "    \"platform\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"agentVersion\": {"]
#[doc = "      \"$ref\": \"#/$defs/Semver\""]
#[doc = "    },"]
#[doc = "    \"capabilities\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/Capability\""]
#[doc = "      },"]
#[doc = "      \"maxItems\": 2,"]
#[doc = "      \"minItems\": 1,"]
#[doc = "      \"uniqueItems\": true"]
#[doc = "    },"]
#[doc = "    \"platform\": {"]
#[doc = "      \"$ref\": \"#/$defs/Platform\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct AgentHelloMessagePayload {
    #[serde(rename = "agentVersion")]
    pub agent_version: Semver,
    pub capabilities: Vec<Capability>,
    pub platform: Platform,
}
impl AgentHelloMessagePayload {
    pub fn builder() -> builder::AgentHelloMessagePayload {
        Default::default()
    }
}
#[doc = "`AgentHelloMessageProtocolVersion`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"integer\","]
#[doc = "  \"enum\": ["]
#[doc = "    1"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct AgentHelloMessageProtocolVersion(i64);
impl ::std::ops::Deref for AgentHelloMessageProtocolVersion {
    type Target = i64;
    fn deref(&self) -> &i64 {
        &self.0
    }
}
impl ::std::convert::From<AgentHelloMessageProtocolVersion> for i64 {
    fn from(value: AgentHelloMessageProtocolVersion) -> Self {
        value.0
    }
}
impl ::std::convert::TryFrom<i64> for AgentHelloMessageProtocolVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: i64) -> ::std::result::Result<Self, self::error::ConversionError> {
        if ![1_i64].contains(&value) {
            Err("invalid value".into())
        } else {
            Ok(Self(value))
        }
    }
}
impl<'de> ::serde::Deserialize<'de> for AgentHelloMessageProtocolVersion {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        Self::try_from(<i64>::deserialize(deserializer)?)
            .map_err(|e| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[doc = "`AgentHelloMessageType`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"agent.hello\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum AgentHelloMessageType {
    #[serde(rename = "agent.hello")]
    AgentHello,
}
impl ::std::fmt::Display for AgentHelloMessageType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::AgentHello => f.write_str("agent.hello"),
        }
    }
}
impl ::std::str::FromStr for AgentHelloMessageType {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "agent.hello" => Ok(Self::AgentHello),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for AgentHelloMessageType {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for AgentHelloMessageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for AgentHelloMessageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`Capability`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"Capability\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"terminal\","]
#[doc = "    \"file-transfer\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum Capability {
    #[serde(rename = "terminal")]
    Terminal,
    #[serde(rename = "file-transfer")]
    FileTransfer,
}
impl ::std::fmt::Display for Capability {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Terminal => f.write_str("terminal"),
            Self::FileTransfer => f.write_str("file-transfer"),
        }
    }
}
impl ::std::str::FromStr for Capability {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "terminal" => Ok(Self::Terminal),
            "file-transfer" => Ok(Self::FileTransfer),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for Capability {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for Capability {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for Capability {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`Cols`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"Cols\","]
#[doc = "  \"type\": \"integer\","]
#[doc = "  \"maximum\": 1000.0,"]
#[doc = "  \"minimum\": 1.0"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct Cols(pub ::std::num::NonZeroU64);
impl ::std::ops::Deref for Cols {
    type Target = ::std::num::NonZeroU64;
    fn deref(&self) -> &::std::num::NonZeroU64 {
        &self.0
    }
}
impl ::std::convert::From<Cols> for ::std::num::NonZeroU64 {
    fn from(value: Cols) -> Self {
        value.0
    }
}
impl ::std::convert::From<::std::num::NonZeroU64> for Cols {
    fn from(value: ::std::num::NonZeroU64) -> Self {
        Self(value)
    }
}
impl ::std::str::FromStr for Cols {
    type Err = <::std::num::NonZeroU64 as ::std::str::FromStr>::Err;
    fn from_str(value: &str) -> ::std::result::Result<Self, Self::Err> {
        Ok(Self(value.parse()?))
    }
}
impl ::std::convert::TryFrom<&str> for Cols {
    type Error = <::std::num::NonZeroU64 as ::std::str::FromStr>::Err;
    fn try_from(value: &str) -> ::std::result::Result<Self, Self::Error> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<String> for Cols {
    type Error = <::std::num::NonZeroU64 as ::std::str::FromStr>::Err;
    fn try_from(value: String) -> ::std::result::Result<Self, Self::Error> {
        value.parse()
    }
}
impl ::std::fmt::Display for Cols {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        self.0.fmt(f)
    }
}
#[doc = "GENERATED bundle of protocol/schema/. Do not edit; run `npm run bundle` in protocol/."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"$id\": \"https://termy.dev/protocol/1/protocol.bundle.schema.json\","]
#[doc = "  \"title\": \"ControlMessage\","]
#[doc = "  \"description\": \"GENERATED bundle of protocol/schema/. Do not edit; run `npm run bundle` in protocol/.\","]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/AgentHeartbeatMessage\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/AgentHelloMessage\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/AgentHelloAckMessage\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/TerminalCloseMessage\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/TerminalErrorMessage\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/TerminalOpenMessage\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/TerminalOpenedMessage\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/TerminalResizeMessage\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/TerminalShellEventMessage\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/TransferAbortMessage\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/TransferAcceptedMessage\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/TransferCompleteMessage\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/TransferCreditMessage\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/TransferFileEndMessage\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/TransferResultMessage\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/TransferStartMessage\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ControlMessage {
    AgentHeartbeatMessage(AgentHeartbeatMessage),
    AgentHelloMessage(AgentHelloMessage),
    AgentHelloAckMessage(AgentHelloAckMessage),
    TerminalCloseMessage(TerminalCloseMessage),
    TerminalErrorMessage(TerminalErrorMessage),
    TerminalOpenMessage(TerminalOpenMessage),
    TerminalOpenedMessage(TerminalOpenedMessage),
    TerminalResizeMessage(TerminalResizeMessage),
    TerminalShellEventMessage(TerminalShellEventMessage),
    TransferAbortMessage(TransferAbortMessage),
    TransferAcceptedMessage(TransferAcceptedMessage),
    TransferCompleteMessage(TransferCompleteMessage),
    TransferCreditMessage(TransferCreditMessage),
    TransferFileEndMessage(TransferFileEndMessage),
    TransferResultMessage(TransferResultMessage),
    TransferStartMessage(TransferStartMessage),
}
impl ::std::convert::From<AgentHeartbeatMessage> for ControlMessage {
    fn from(value: AgentHeartbeatMessage) -> Self {
        Self::AgentHeartbeatMessage(value)
    }
}
impl ::std::convert::From<AgentHelloMessage> for ControlMessage {
    fn from(value: AgentHelloMessage) -> Self {
        Self::AgentHelloMessage(value)
    }
}
impl ::std::convert::From<AgentHelloAckMessage> for ControlMessage {
    fn from(value: AgentHelloAckMessage) -> Self {
        Self::AgentHelloAckMessage(value)
    }
}
impl ::std::convert::From<TerminalCloseMessage> for ControlMessage {
    fn from(value: TerminalCloseMessage) -> Self {
        Self::TerminalCloseMessage(value)
    }
}
impl ::std::convert::From<TerminalErrorMessage> for ControlMessage {
    fn from(value: TerminalErrorMessage) -> Self {
        Self::TerminalErrorMessage(value)
    }
}
impl ::std::convert::From<TerminalOpenMessage> for ControlMessage {
    fn from(value: TerminalOpenMessage) -> Self {
        Self::TerminalOpenMessage(value)
    }
}
impl ::std::convert::From<TerminalOpenedMessage> for ControlMessage {
    fn from(value: TerminalOpenedMessage) -> Self {
        Self::TerminalOpenedMessage(value)
    }
}
impl ::std::convert::From<TerminalResizeMessage> for ControlMessage {
    fn from(value: TerminalResizeMessage) -> Self {
        Self::TerminalResizeMessage(value)
    }
}
impl ::std::convert::From<TerminalShellEventMessage> for ControlMessage {
    fn from(value: TerminalShellEventMessage) -> Self {
        Self::TerminalShellEventMessage(value)
    }
}
impl ::std::convert::From<TransferAbortMessage> for ControlMessage {
    fn from(value: TransferAbortMessage) -> Self {
        Self::TransferAbortMessage(value)
    }
}
impl ::std::convert::From<TransferAcceptedMessage> for ControlMessage {
    fn from(value: TransferAcceptedMessage) -> Self {
        Self::TransferAcceptedMessage(value)
    }
}
impl ::std::convert::From<TransferCompleteMessage> for ControlMessage {
    fn from(value: TransferCompleteMessage) -> Self {
        Self::TransferCompleteMessage(value)
    }
}
impl ::std::convert::From<TransferCreditMessage> for ControlMessage {
    fn from(value: TransferCreditMessage) -> Self {
        Self::TransferCreditMessage(value)
    }
}
impl ::std::convert::From<TransferFileEndMessage> for ControlMessage {
    fn from(value: TransferFileEndMessage) -> Self {
        Self::TransferFileEndMessage(value)
    }
}
impl ::std::convert::From<TransferResultMessage> for ControlMessage {
    fn from(value: TransferResultMessage) -> Self {
        Self::TransferResultMessage(value)
    }
}
impl ::std::convert::From<TransferStartMessage> for ControlMessage {
    fn from(value: TransferStartMessage) -> Self {
        Self::TransferStartMessage(value)
    }
}
#[doc = "UTC RFC 3339 timestamp."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"DateTime\","]
#[doc = "  \"description\": \"UTC RFC 3339 timestamp.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"format\": \"date-time\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct DateTime(pub ::chrono::DateTime<::chrono::offset::Utc>);
impl ::std::ops::Deref for DateTime {
    type Target = ::chrono::DateTime<::chrono::offset::Utc>;
    fn deref(&self) -> &::chrono::DateTime<::chrono::offset::Utc> {
        &self.0
    }
}
impl ::std::convert::From<DateTime> for ::chrono::DateTime<::chrono::offset::Utc> {
    fn from(value: DateTime) -> Self {
        value.0
    }
}
impl ::std::convert::From<::chrono::DateTime<::chrono::offset::Utc>> for DateTime {
    fn from(value: ::chrono::DateTime<::chrono::offset::Utc>) -> Self {
        Self(value)
    }
}
impl ::std::str::FromStr for DateTime {
    type Err = <::chrono::DateTime<::chrono::offset::Utc> as ::std::str::FromStr>::Err;
    fn from_str(value: &str) -> ::std::result::Result<Self, Self::Err> {
        Ok(Self(value.parse()?))
    }
}
impl ::std::convert::TryFrom<&str> for DateTime {
    type Error = <::chrono::DateTime<::chrono::offset::Utc> as ::std::str::FromStr>::Err;
    fn try_from(value: &str) -> ::std::result::Result<Self, Self::Error> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<String> for DateTime {
    type Error = <::chrono::DateTime<::chrono::offset::Utc> as ::std::str::FromStr>::Err;
    fn try_from(value: String) -> ::std::result::Result<Self, Self::Error> {
        value.parse()
    }
}
impl ::std::fmt::Display for DateTime {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        self.0.fmt(f)
    }
}
#[doc = "`ErrorCode`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"ErrorCode\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"AUTH_EXPIRED\","]
#[doc = "    \"AUTH_INVALID\","]
#[doc = "    \"PAIRING_CODE_INVALID\","]
#[doc = "    \"DEVICE_FORBIDDEN\","]
#[doc = "    \"DEVICE_OFFLINE\","]
#[doc = "    \"DEVICE_BUSY\","]
#[doc = "    \"SHELL_START_FAILED\","]
#[doc = "    \"RELAY_DISCONNECTED\","]
#[doc = "    \"INVALID_DROP\","]
#[doc = "    \"INVALID_PATH\","]
#[doc = "    \"WRITE_FAILED\","]
#[doc = "    \"TRANSFER_FAILED\","]
#[doc = "    \"QUOTA_EXCEEDED\","]
#[doc = "    \"RATE_LIMITED\","]
#[doc = "    \"BACKPRESSURE_LIMIT\","]
#[doc = "    \"SESSION_TIMEOUT\","]
#[doc = "    \"PROTOCOL_ERROR\","]
#[doc = "    \"INTERNAL_ERROR\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum ErrorCode {
    #[serde(rename = "AUTH_EXPIRED")]
    AuthExpired,
    #[serde(rename = "AUTH_INVALID")]
    AuthInvalid,
    #[serde(rename = "PAIRING_CODE_INVALID")]
    PairingCodeInvalid,
    #[serde(rename = "DEVICE_FORBIDDEN")]
    DeviceForbidden,
    #[serde(rename = "DEVICE_OFFLINE")]
    DeviceOffline,
    #[serde(rename = "DEVICE_BUSY")]
    DeviceBusy,
    #[serde(rename = "SHELL_START_FAILED")]
    ShellStartFailed,
    #[serde(rename = "RELAY_DISCONNECTED")]
    RelayDisconnected,
    #[serde(rename = "INVALID_DROP")]
    InvalidDrop,
    #[serde(rename = "INVALID_PATH")]
    InvalidPath,
    #[serde(rename = "WRITE_FAILED")]
    WriteFailed,
    #[serde(rename = "TRANSFER_FAILED")]
    TransferFailed,
    #[serde(rename = "QUOTA_EXCEEDED")]
    QuotaExceeded,
    #[serde(rename = "RATE_LIMITED")]
    RateLimited,
    #[serde(rename = "BACKPRESSURE_LIMIT")]
    BackpressureLimit,
    #[serde(rename = "SESSION_TIMEOUT")]
    SessionTimeout,
    #[serde(rename = "PROTOCOL_ERROR")]
    ProtocolError,
    #[serde(rename = "INTERNAL_ERROR")]
    InternalError,
}
impl ::std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::AuthExpired => f.write_str("AUTH_EXPIRED"),
            Self::AuthInvalid => f.write_str("AUTH_INVALID"),
            Self::PairingCodeInvalid => f.write_str("PAIRING_CODE_INVALID"),
            Self::DeviceForbidden => f.write_str("DEVICE_FORBIDDEN"),
            Self::DeviceOffline => f.write_str("DEVICE_OFFLINE"),
            Self::DeviceBusy => f.write_str("DEVICE_BUSY"),
            Self::ShellStartFailed => f.write_str("SHELL_START_FAILED"),
            Self::RelayDisconnected => f.write_str("RELAY_DISCONNECTED"),
            Self::InvalidDrop => f.write_str("INVALID_DROP"),
            Self::InvalidPath => f.write_str("INVALID_PATH"),
            Self::WriteFailed => f.write_str("WRITE_FAILED"),
            Self::TransferFailed => f.write_str("TRANSFER_FAILED"),
            Self::QuotaExceeded => f.write_str("QUOTA_EXCEEDED"),
            Self::RateLimited => f.write_str("RATE_LIMITED"),
            Self::BackpressureLimit => f.write_str("BACKPRESSURE_LIMIT"),
            Self::SessionTimeout => f.write_str("SESSION_TIMEOUT"),
            Self::ProtocolError => f.write_str("PROTOCOL_ERROR"),
            Self::InternalError => f.write_str("INTERNAL_ERROR"),
        }
    }
}
impl ::std::str::FromStr for ErrorCode {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "AUTH_EXPIRED" => Ok(Self::AuthExpired),
            "AUTH_INVALID" => Ok(Self::AuthInvalid),
            "PAIRING_CODE_INVALID" => Ok(Self::PairingCodeInvalid),
            "DEVICE_FORBIDDEN" => Ok(Self::DeviceForbidden),
            "DEVICE_OFFLINE" => Ok(Self::DeviceOffline),
            "DEVICE_BUSY" => Ok(Self::DeviceBusy),
            "SHELL_START_FAILED" => Ok(Self::ShellStartFailed),
            "RELAY_DISCONNECTED" => Ok(Self::RelayDisconnected),
            "INVALID_DROP" => Ok(Self::InvalidDrop),
            "INVALID_PATH" => Ok(Self::InvalidPath),
            "WRITE_FAILED" => Ok(Self::WriteFailed),
            "TRANSFER_FAILED" => Ok(Self::TransferFailed),
            "QUOTA_EXCEEDED" => Ok(Self::QuotaExceeded),
            "RATE_LIMITED" => Ok(Self::RateLimited),
            "BACKPRESSURE_LIMIT" => Ok(Self::BackpressureLimit),
            "SESSION_TIMEOUT" => Ok(Self::SessionTimeout),
            "PROTOCOL_ERROR" => Ok(Self::ProtocolError),
            "INTERNAL_ERROR" => Ok(Self::InternalError),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for ErrorCode {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ErrorCode {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ErrorCode {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`FileEntry`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"FileEntry\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"index\","]
#[doc = "    \"relativePath\","]
#[doc = "    \"size\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"index\": {"]
#[doc = "      \"$ref\": \"#/$defs/FileIndex\""]
#[doc = "    },"]
#[doc = "    \"relativePath\": {"]
#[doc = "      \"$ref\": \"#/$defs/SafeRelativePath\""]
#[doc = "    },"]
#[doc = "    \"size\": {"]
#[doc = "      \"$ref\": \"#/$defs/FileSize\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct FileEntry {
    pub index: FileIndex,
    #[serde(rename = "relativePath")]
    pub relative_path: SafeRelativePath,
    pub size: FileSize,
}
impl FileEntry {
    pub fn builder() -> builder::FileEntry {
        Default::default()
    }
}
#[doc = "`FileIndex`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"FileIndex\","]
#[doc = "  \"type\": \"integer\","]
#[doc = "  \"maximum\": 255.0,"]
#[doc = "  \"minimum\": 0.0"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct FileIndex(pub u8);
impl ::std::ops::Deref for FileIndex {
    type Target = u8;
    fn deref(&self) -> &u8 {
        &self.0
    }
}
impl ::std::convert::From<FileIndex> for u8 {
    fn from(value: FileIndex) -> Self {
        value.0
    }
}
impl ::std::convert::From<u8> for FileIndex {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl ::std::str::FromStr for FileIndex {
    type Err = <u8 as ::std::str::FromStr>::Err;
    fn from_str(value: &str) -> ::std::result::Result<Self, Self::Err> {
        Ok(Self(value.parse()?))
    }
}
impl ::std::convert::TryFrom<&str> for FileIndex {
    type Error = <u8 as ::std::str::FromStr>::Err;
    fn try_from(value: &str) -> ::std::result::Result<Self, Self::Error> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<String> for FileIndex {
    type Error = <u8 as ::std::str::FromStr>::Err;
    fn try_from(value: String) -> ::std::result::Result<Self, Self::Error> {
        value.parse()
    }
}
impl ::std::fmt::Display for FileIndex {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        self.0.fmt(f)
    }
}
#[doc = "Per-file byte limit, 64 MiB (doc 4.12 / 8.4)."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"FileSize\","]
#[doc = "  \"description\": \"Per-file byte limit, 64 MiB (doc 4.12 / 8.4).\","]
#[doc = "  \"type\": \"integer\","]
#[doc = "  \"maximum\": 67108864.0,"]
#[doc = "  \"minimum\": 0.0"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct FileSize(pub i64);
impl ::std::ops::Deref for FileSize {
    type Target = i64;
    fn deref(&self) -> &i64 {
        &self.0
    }
}
impl ::std::convert::From<FileSize> for i64 {
    fn from(value: FileSize) -> Self {
        value.0
    }
}
impl ::std::convert::From<i64> for FileSize {
    fn from(value: i64) -> Self {
        Self(value)
    }
}
impl ::std::str::FromStr for FileSize {
    type Err = <i64 as ::std::str::FromStr>::Err;
    fn from_str(value: &str) -> ::std::result::Result<Self, Self::Err> {
        Ok(Self(value.parse()?))
    }
}
impl ::std::convert::TryFrom<&str> for FileSize {
    type Error = <i64 as ::std::str::FromStr>::Err;
    fn try_from(value: &str) -> ::std::result::Result<Self, Self::Error> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<String> for FileSize {
    type Error = <i64 as ::std::str::FromStr>::Err;
    fn try_from(value: String) -> ::std::result::Result<Self, Self::Error> {
        value.parse()
    }
}
impl ::std::fmt::Display for FileSize {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        self.0.fmt(f)
    }
}
#[doc = "`Platform`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"Platform\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"windows-x64\","]
#[doc = "    \"ubuntu-x64\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum Platform {
    #[serde(rename = "windows-x64")]
    WindowsX64,
    #[serde(rename = "ubuntu-x64")]
    UbuntuX64,
}
impl ::std::fmt::Display for Platform {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::WindowsX64 => f.write_str("windows-x64"),
            Self::UbuntuX64 => f.write_str("ubuntu-x64"),
        }
    }
}
impl ::std::str::FromStr for Platform {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "windows-x64" => Ok(Self::WindowsX64),
            "ubuntu-x64" => Ok(Self::UbuntuX64),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for Platform {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for Platform {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for Platform {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`Rows`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"Rows\","]
#[doc = "  \"type\": \"integer\","]
#[doc = "  \"maximum\": 500.0,"]
#[doc = "  \"minimum\": 1.0"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct Rows(pub ::std::num::NonZeroU64);
impl ::std::ops::Deref for Rows {
    type Target = ::std::num::NonZeroU64;
    fn deref(&self) -> &::std::num::NonZeroU64 {
        &self.0
    }
}
impl ::std::convert::From<Rows> for ::std::num::NonZeroU64 {
    fn from(value: Rows) -> Self {
        value.0
    }
}
impl ::std::convert::From<::std::num::NonZeroU64> for Rows {
    fn from(value: ::std::num::NonZeroU64) -> Self {
        Self(value)
    }
}
impl ::std::str::FromStr for Rows {
    type Err = <::std::num::NonZeroU64 as ::std::str::FromStr>::Err;
    fn from_str(value: &str) -> ::std::result::Result<Self, Self::Err> {
        Ok(Self(value.parse()?))
    }
}
impl ::std::convert::TryFrom<&str> for Rows {
    type Error = <::std::num::NonZeroU64 as ::std::str::FromStr>::Err;
    fn try_from(value: &str) -> ::std::result::Result<Self, Self::Error> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<String> for Rows {
    type Error = <::std::num::NonZeroU64 as ::std::str::FromStr>::Err;
    fn try_from(value: String) -> ::std::result::Result<Self, Self::Error> {
        value.parse()
    }
}
impl ::std::fmt::Display for Rows {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        self.0.fmt(f)
    }
}
#[doc = "Vault-relative path. Always '/'-separated. Structural safety (no '..', no drive letter, no UNC root, no absolute form, no empty segment, no NUL) is enforced in code per doc 10.3; maxLength here counts UTF-16 code units and is an upper bound only, the 1024-byte UTF-8 limit is enforced in code."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"SafeRelativePath\","]
#[doc = "  \"description\": \"Vault-relative path. Always '/'-separated. Structural safety (no '..', no drive letter, no UNC root, no absolute form, no empty segment, no NUL) is enforced in code per doc 10.3; maxLength here counts UTF-16 code units and is an upper bound only, the 1024-byte UTF-8 limit is enforced in code.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 1024,"]
#[doc = "  \"minLength\": 1,"]
#[doc = "  \"pattern\": \"^(?!/)(?![A-Za-z]:)(?!.*\\\\\\\\)(?!.*//)(?!.*(^|/)\\\\.\\\\.(/|$))(?!.*(^|/)\\\\.(/|$))[^\\\\u0000]*[^\\\\u0000/]$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct SafeRelativePath(::std::string::String);
impl ::std::ops::Deref for SafeRelativePath {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<SafeRelativePath> for ::std::string::String {
    fn from(value: SafeRelativePath) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for SafeRelativePath {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 1024usize {
            return Err("longer than 1024 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> = ::std::sync::LazyLock::new(
            || {
                :: regress :: Regex :: new ("^(?!/)(?![A-Za-z]:)(?!.*\\\\)(?!.*//)(?!.*(^|/)\\.\\.(/|$))(?!.*(^|/)\\.(/|$))[^\\u0000]*[^\\u0000/]$") . unwrap ()
            },
        );
        if PATTERN.find(value).is_none() {
            return Err ("doesn't match pattern \"^(?!/)(?![A-Za-z]:)(?!.*\\\\)(?!.*//)(?!.*(^|/)\\.\\.(/|$))(?!.*(^|/)\\.(/|$))[^\\u0000]*[^\\u0000/]$\"" . into ()) ;
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for SafeRelativePath {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for SafeRelativePath {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for SafeRelativePath {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for SafeRelativePath {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
#[doc = "`Semver`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"Semver\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^(0|[1-9]\\\\d*)\\\\.(0|[1-9]\\\\d*)\\\\.(0|[1-9]\\\\d*)(?:-[0-9A-Za-z-.]+)?(?:\\\\+[0-9A-Za-z-.]+)?$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct Semver(::std::string::String);
impl ::std::ops::Deref for Semver {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<Semver> for ::std::string::String {
    fn from(value: Semver) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for Semver {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> = ::std::sync::LazyLock::new(
            || {
                :: regress :: Regex :: new ("^(0|[1-9]\\d*)\\.(0|[1-9]\\d*)\\.(0|[1-9]\\d*)(?:-[0-9A-Za-z-.]+)?(?:\\+[0-9A-Za-z-.]+)?$") . unwrap ()
            },
        );
        if PATTERN.find(value).is_none() {
            return Err ("doesn't match pattern \"^(0|[1-9]\\d*)\\.(0|[1-9]\\d*)\\.(0|[1-9]\\d*)(?:-[0-9A-Za-z-.]+)?(?:\\+[0-9A-Za-z-.]+)?$\"" . into ()) ;
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for Semver {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for Semver {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for Semver {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for Semver {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
#[doc = "Mirrors Termy's local ShellEventType exactly, so the plugin can hand payload straight to its existing shell-event handler."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"ShellEventName\","]
#[doc = "  \"description\": \"Mirrors Termy's local ShellEventType exactly, so the plugin can hand payload straight to its existing shell-event handler.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"prompt_start\","]
#[doc = "    \"command_start\","]
#[doc = "    \"command_executed\","]
#[doc = "    \"command_end\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum ShellEventName {
    #[serde(rename = "prompt_start")]
    PromptStart,
    #[serde(rename = "command_start")]
    CommandStart,
    #[serde(rename = "command_executed")]
    CommandExecuted,
    #[serde(rename = "command_end")]
    CommandEnd,
}
impl ::std::fmt::Display for ShellEventName {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::PromptStart => f.write_str("prompt_start"),
            Self::CommandStart => f.write_str("command_start"),
            Self::CommandExecuted => f.write_str("command_executed"),
            Self::CommandEnd => f.write_str("command_end"),
        }
    }
}
impl ::std::str::FromStr for ShellEventName {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "prompt_start" => Ok(Self::PromptStart),
            "command_start" => Ok(Self::CommandStart),
            "command_executed" => Ok(Self::CommandExecuted),
            "command_end" => Ok(Self::CommandEnd),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for ShellEventName {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ShellEventName {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ShellEventName {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "Which shell-integration sequence produced the event, mirroring Termy's local ShellEventSource."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"ShellEventSource\","]
#[doc = "  \"description\": \"Which shell-integration sequence produced the event, mirroring Termy's local ShellEventSource.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"osc133\","]
#[doc = "    \"osc633\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum ShellEventSource {
    #[serde(rename = "osc133")]
    Osc133,
    #[serde(rename = "osc633")]
    Osc633,
}
impl ::std::fmt::Display for ShellEventSource {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::Osc133 => f.write_str("osc133"),
            Self::Osc633 => f.write_str("osc633"),
        }
    }
}
impl ::std::str::FromStr for ShellEventSource {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "osc133" => Ok(Self::Osc133),
            "osc633" => Ok(Self::Osc633),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for ShellEventSource {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for ShellEventSource {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for ShellEventSource {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "Plugin sends it to end a session (reason=user). Agent sends it when the shell exits (reason=shell_exited, exitCode set) or the peer went away."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"TerminalCloseMessage\","]
#[doc = "  \"description\": \"Plugin sends it to end a session (reason=user). Agent sends it when the shell exits (reason=shell_exited, exitCode set) or the peer went away.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"deviceId\","]
#[doc = "    \"payload\","]
#[doc = "    \"protocolVersion\","]
#[doc = "    \"requestId\","]
#[doc = "    \"sessionId\","]
#[doc = "    \"type\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"deviceId\": {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    },"]
#[doc = "    \"payload\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"exitCode\","]
#[doc = "        \"reason\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"exitCode\": {"]
#[doc = "          \"oneOf\": ["]
#[doc = "            {"]
#[doc = "              \"type\": \"integer\","]
#[doc = "              \"maximum\": 2147483647.0,"]
#[doc = "              \"minimum\": -2147483648.0"]
#[doc = "            },"]
#[doc = "            {"]
#[doc = "              \"type\": \"null\""]
#[doc = "            }"]
#[doc = "          ]"]
#[doc = "        },"]
#[doc = "        \"reason\": {"]
#[doc = "          \"$ref\": \"#/$defs/TerminalCloseReason\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"protocolVersion\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"enum\": ["]
#[doc = "        1"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"requestId\": {"]
#[doc = "      \"type\": \"null\""]
#[doc = "    },"]
#[doc = "    \"sessionId\": {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    },"]
#[doc = "    \"type\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"terminal.close\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TerminalCloseMessage {
    #[serde(rename = "deviceId")]
    pub device_id: Uuid,
    pub payload: TerminalCloseMessagePayload,
    #[serde(rename = "protocolVersion")]
    pub protocol_version: TerminalCloseMessageProtocolVersion,
    #[serde(rename = "requestId")]
    pub request_id: (),
    #[serde(rename = "sessionId")]
    pub session_id: Uuid,
    #[serde(rename = "type")]
    pub type_: TerminalCloseMessageType,
}
impl TerminalCloseMessage {
    pub fn builder() -> builder::TerminalCloseMessage {
        Default::default()
    }
}
#[doc = "`TerminalCloseMessagePayload`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"exitCode\","]
#[doc = "    \"reason\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"exitCode\": {"]
#[doc = "      \"oneOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"maximum\": 2147483647.0,"]
#[doc = "          \"minimum\": -2147483648.0"]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"reason\": {"]
#[doc = "      \"$ref\": \"#/$defs/TerminalCloseReason\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TerminalCloseMessagePayload {
    #[serde(rename = "exitCode")]
    pub exit_code: ::std::option::Option<i32>,
    pub reason: TerminalCloseReason,
}
impl TerminalCloseMessagePayload {
    pub fn builder() -> builder::TerminalCloseMessagePayload {
        Default::default()
    }
}
#[doc = "`TerminalCloseMessageProtocolVersion`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"integer\","]
#[doc = "  \"enum\": ["]
#[doc = "    1"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct TerminalCloseMessageProtocolVersion(i64);
impl ::std::ops::Deref for TerminalCloseMessageProtocolVersion {
    type Target = i64;
    fn deref(&self) -> &i64 {
        &self.0
    }
}
impl ::std::convert::From<TerminalCloseMessageProtocolVersion> for i64 {
    fn from(value: TerminalCloseMessageProtocolVersion) -> Self {
        value.0
    }
}
impl ::std::convert::TryFrom<i64> for TerminalCloseMessageProtocolVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: i64) -> ::std::result::Result<Self, self::error::ConversionError> {
        if ![1_i64].contains(&value) {
            Err("invalid value".into())
        } else {
            Ok(Self(value))
        }
    }
}
impl<'de> ::serde::Deserialize<'de> for TerminalCloseMessageProtocolVersion {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        Self::try_from(<i64>::deserialize(deserializer)?)
            .map_err(|e| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[doc = "`TerminalCloseMessageType`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"terminal.close\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum TerminalCloseMessageType {
    #[serde(rename = "terminal.close")]
    TerminalClose,
}
impl ::std::fmt::Display for TerminalCloseMessageType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::TerminalClose => f.write_str("terminal.close"),
        }
    }
}
impl ::std::str::FromStr for TerminalCloseMessageType {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "terminal.close" => Ok(Self::TerminalClose),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for TerminalCloseMessageType {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TerminalCloseMessageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TerminalCloseMessageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`TerminalCloseReason`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"TerminalCloseReason\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"enum\": ["]
#[doc = "    \"user\","]
#[doc = "    \"peer_disconnected\","]
#[doc = "    \"shell_exited\","]
#[doc = "    \"error\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum TerminalCloseReason {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "peer_disconnected")]
    PeerDisconnected,
    #[serde(rename = "shell_exited")]
    ShellExited,
    #[serde(rename = "error")]
    Error,
}
impl ::std::fmt::Display for TerminalCloseReason {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::User => f.write_str("user"),
            Self::PeerDisconnected => f.write_str("peer_disconnected"),
            Self::ShellExited => f.write_str("shell_exited"),
            Self::Error => f.write_str("error"),
        }
    }
}
impl ::std::str::FromStr for TerminalCloseReason {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "user" => Ok(Self::User),
            "peer_disconnected" => Ok(Self::PeerDisconnected),
            "shell_exited" => Ok(Self::ShellExited),
            "error" => Ok(Self::Error),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for TerminalCloseReason {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TerminalCloseReason {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TerminalCloseReason {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "sessionId is null when the failure happens before a session exists (DEVICE_OFFLINE, DEVICE_BUSY, SHELL_START_FAILED in response to terminal.open); requestId echoes the failed request when there is one. message must be redacted per doc 8.8.6."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"TerminalErrorMessage\","]
#[doc = "  \"description\": \"sessionId is null when the failure happens before a session exists (DEVICE_OFFLINE, DEVICE_BUSY, SHELL_START_FAILED in response to terminal.open); requestId echoes the failed request when there is one. message must be redacted per doc 8.8.6.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"deviceId\","]
#[doc = "    \"payload\","]
#[doc = "    \"protocolVersion\","]
#[doc = "    \"requestId\","]
#[doc = "    \"sessionId\","]
#[doc = "    \"type\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"deviceId\": {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    },"]
#[doc = "    \"payload\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"code\","]
#[doc = "        \"message\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"code\": {"]
#[doc = "          \"$ref\": \"#/$defs/ErrorCode\""]
#[doc = "        },"]
#[doc = "        \"message\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"maxLength\": 512,"]
#[doc = "          \"minLength\": 1"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"protocolVersion\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"enum\": ["]
#[doc = "        1"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"requestId\": {"]
#[doc = "      \"$ref\": \"#/$defs/UuidOrNull\""]
#[doc = "    },"]
#[doc = "    \"sessionId\": {"]
#[doc = "      \"$ref\": \"#/$defs/UuidOrNull\""]
#[doc = "    },"]
#[doc = "    \"type\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"terminal.error\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TerminalErrorMessage {
    #[serde(rename = "deviceId")]
    pub device_id: Uuid,
    pub payload: TerminalErrorMessagePayload,
    #[serde(rename = "protocolVersion")]
    pub protocol_version: TerminalErrorMessageProtocolVersion,
    #[serde(rename = "requestId")]
    pub request_id: UuidOrNull,
    #[serde(rename = "sessionId")]
    pub session_id: UuidOrNull,
    #[serde(rename = "type")]
    pub type_: TerminalErrorMessageType,
}
impl TerminalErrorMessage {
    pub fn builder() -> builder::TerminalErrorMessage {
        Default::default()
    }
}
#[doc = "`TerminalErrorMessagePayload`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"code\","]
#[doc = "    \"message\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"code\": {"]
#[doc = "      \"$ref\": \"#/$defs/ErrorCode\""]
#[doc = "    },"]
#[doc = "    \"message\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 512,"]
#[doc = "      \"minLength\": 1"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TerminalErrorMessagePayload {
    pub code: ErrorCode,
    pub message: TerminalErrorMessagePayloadMessage,
}
impl TerminalErrorMessagePayload {
    pub fn builder() -> builder::TerminalErrorMessagePayload {
        Default::default()
    }
}
#[doc = "`TerminalErrorMessagePayloadMessage`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 512,"]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct TerminalErrorMessagePayloadMessage(::std::string::String);
impl ::std::ops::Deref for TerminalErrorMessagePayloadMessage {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<TerminalErrorMessagePayloadMessage> for ::std::string::String {
    fn from(value: TerminalErrorMessagePayloadMessage) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for TerminalErrorMessagePayloadMessage {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 512usize {
            return Err("longer than 512 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for TerminalErrorMessagePayloadMessage {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TerminalErrorMessagePayloadMessage {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TerminalErrorMessagePayloadMessage {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for TerminalErrorMessagePayloadMessage {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
#[doc = "`TerminalErrorMessageProtocolVersion`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"integer\","]
#[doc = "  \"enum\": ["]
#[doc = "    1"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct TerminalErrorMessageProtocolVersion(i64);
impl ::std::ops::Deref for TerminalErrorMessageProtocolVersion {
    type Target = i64;
    fn deref(&self) -> &i64 {
        &self.0
    }
}
impl ::std::convert::From<TerminalErrorMessageProtocolVersion> for i64 {
    fn from(value: TerminalErrorMessageProtocolVersion) -> Self {
        value.0
    }
}
impl ::std::convert::TryFrom<i64> for TerminalErrorMessageProtocolVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: i64) -> ::std::result::Result<Self, self::error::ConversionError> {
        if ![1_i64].contains(&value) {
            Err("invalid value".into())
        } else {
            Ok(Self(value))
        }
    }
}
impl<'de> ::serde::Deserialize<'de> for TerminalErrorMessageProtocolVersion {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        Self::try_from(<i64>::deserialize(deserializer)?)
            .map_err(|e| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[doc = "`TerminalErrorMessageType`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"terminal.error\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum TerminalErrorMessageType {
    #[serde(rename = "terminal.error")]
    TerminalError,
}
impl ::std::fmt::Display for TerminalErrorMessageType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::TerminalError => f.write_str("terminal.error"),
        }
    }
}
impl ::std::str::FromStr for TerminalErrorMessageType {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "terminal.error" => Ok(Self::TerminalError),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for TerminalErrorMessageType {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TerminalErrorMessageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TerminalErrorMessageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`TerminalOpenMessage`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"TerminalOpenMessage\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"deviceId\","]
#[doc = "    \"payload\","]
#[doc = "    \"protocolVersion\","]
#[doc = "    \"requestId\","]
#[doc = "    \"sessionId\","]
#[doc = "    \"type\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"deviceId\": {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    },"]
#[doc = "    \"payload\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"cols\","]
#[doc = "        \"rows\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"cols\": {"]
#[doc = "          \"$ref\": \"#/$defs/Cols\""]
#[doc = "        },"]
#[doc = "        \"rows\": {"]
#[doc = "          \"$ref\": \"#/$defs/Rows\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"protocolVersion\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"enum\": ["]
#[doc = "        1"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"requestId\": {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    },"]
#[doc = "    \"sessionId\": {"]
#[doc = "      \"type\": \"null\""]
#[doc = "    },"]
#[doc = "    \"type\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"terminal.open\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TerminalOpenMessage {
    #[serde(rename = "deviceId")]
    pub device_id: Uuid,
    pub payload: TerminalOpenMessagePayload,
    #[serde(rename = "protocolVersion")]
    pub protocol_version: TerminalOpenMessageProtocolVersion,
    #[serde(rename = "requestId")]
    pub request_id: Uuid,
    #[serde(rename = "sessionId")]
    pub session_id: (),
    #[serde(rename = "type")]
    pub type_: TerminalOpenMessageType,
}
impl TerminalOpenMessage {
    pub fn builder() -> builder::TerminalOpenMessage {
        Default::default()
    }
}
#[doc = "`TerminalOpenMessagePayload`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"cols\","]
#[doc = "    \"rows\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"cols\": {"]
#[doc = "      \"$ref\": \"#/$defs/Cols\""]
#[doc = "    },"]
#[doc = "    \"rows\": {"]
#[doc = "      \"$ref\": \"#/$defs/Rows\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TerminalOpenMessagePayload {
    pub cols: Cols,
    pub rows: Rows,
}
impl TerminalOpenMessagePayload {
    pub fn builder() -> builder::TerminalOpenMessagePayload {
        Default::default()
    }
}
#[doc = "`TerminalOpenMessageProtocolVersion`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"integer\","]
#[doc = "  \"enum\": ["]
#[doc = "    1"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct TerminalOpenMessageProtocolVersion(i64);
impl ::std::ops::Deref for TerminalOpenMessageProtocolVersion {
    type Target = i64;
    fn deref(&self) -> &i64 {
        &self.0
    }
}
impl ::std::convert::From<TerminalOpenMessageProtocolVersion> for i64 {
    fn from(value: TerminalOpenMessageProtocolVersion) -> Self {
        value.0
    }
}
impl ::std::convert::TryFrom<i64> for TerminalOpenMessageProtocolVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: i64) -> ::std::result::Result<Self, self::error::ConversionError> {
        if ![1_i64].contains(&value) {
            Err("invalid value".into())
        } else {
            Ok(Self(value))
        }
    }
}
impl<'de> ::serde::Deserialize<'de> for TerminalOpenMessageProtocolVersion {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        Self::try_from(<i64>::deserialize(deserializer)?)
            .map_err(|e| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[doc = "`TerminalOpenMessageType`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"terminal.open\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum TerminalOpenMessageType {
    #[serde(rename = "terminal.open")]
    TerminalOpen,
}
impl ::std::fmt::Display for TerminalOpenMessageType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::TerminalOpen => f.write_str("terminal.open"),
        }
    }
}
impl ::std::str::FromStr for TerminalOpenMessageType {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "terminal.open" => Ok(Self::TerminalOpen),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for TerminalOpenMessageType {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TerminalOpenMessageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TerminalOpenMessageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "Direct response to terminal.open: requestId matches, and the envelope carries the sessionId minted by the Agent (doc 8.8.3)."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"TerminalOpenedMessage\","]
#[doc = "  \"description\": \"Direct response to terminal.open: requestId matches, and the envelope carries the sessionId minted by the Agent (doc 8.8.3).\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"deviceId\","]
#[doc = "    \"payload\","]
#[doc = "    \"protocolVersion\","]
#[doc = "    \"requestId\","]
#[doc = "    \"sessionId\","]
#[doc = "    \"type\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"deviceId\": {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    },"]
#[doc = "    \"payload\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"shell\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"shell\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"maxLength\": 260,"]
#[doc = "          \"minLength\": 1"]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"protocolVersion\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"enum\": ["]
#[doc = "        1"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"requestId\": {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    },"]
#[doc = "    \"sessionId\": {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    },"]
#[doc = "    \"type\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"terminal.opened\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TerminalOpenedMessage {
    #[serde(rename = "deviceId")]
    pub device_id: Uuid,
    pub payload: TerminalOpenedMessagePayload,
    #[serde(rename = "protocolVersion")]
    pub protocol_version: TerminalOpenedMessageProtocolVersion,
    #[serde(rename = "requestId")]
    pub request_id: Uuid,
    #[serde(rename = "sessionId")]
    pub session_id: Uuid,
    #[serde(rename = "type")]
    pub type_: TerminalOpenedMessageType,
}
impl TerminalOpenedMessage {
    pub fn builder() -> builder::TerminalOpenedMessage {
        Default::default()
    }
}
#[doc = "`TerminalOpenedMessagePayload`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"shell\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"shell\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 260,"]
#[doc = "      \"minLength\": 1"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TerminalOpenedMessagePayload {
    pub shell: TerminalOpenedMessagePayloadShell,
}
impl TerminalOpenedMessagePayload {
    pub fn builder() -> builder::TerminalOpenedMessagePayload {
        Default::default()
    }
}
#[doc = "`TerminalOpenedMessagePayloadShell`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 260,"]
#[doc = "  \"minLength\": 1"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct TerminalOpenedMessagePayloadShell(::std::string::String);
impl ::std::ops::Deref for TerminalOpenedMessagePayloadShell {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<TerminalOpenedMessagePayloadShell> for ::std::string::String {
    fn from(value: TerminalOpenedMessagePayloadShell) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for TerminalOpenedMessagePayloadShell {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 260usize {
            return Err("longer than 260 characters".into());
        }
        if value.chars().count() < 1usize {
            return Err("shorter than 1 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for TerminalOpenedMessagePayloadShell {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TerminalOpenedMessagePayloadShell {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TerminalOpenedMessagePayloadShell {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for TerminalOpenedMessagePayloadShell {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
#[doc = "`TerminalOpenedMessageProtocolVersion`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"integer\","]
#[doc = "  \"enum\": ["]
#[doc = "    1"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct TerminalOpenedMessageProtocolVersion(i64);
impl ::std::ops::Deref for TerminalOpenedMessageProtocolVersion {
    type Target = i64;
    fn deref(&self) -> &i64 {
        &self.0
    }
}
impl ::std::convert::From<TerminalOpenedMessageProtocolVersion> for i64 {
    fn from(value: TerminalOpenedMessageProtocolVersion) -> Self {
        value.0
    }
}
impl ::std::convert::TryFrom<i64> for TerminalOpenedMessageProtocolVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: i64) -> ::std::result::Result<Self, self::error::ConversionError> {
        if ![1_i64].contains(&value) {
            Err("invalid value".into())
        } else {
            Ok(Self(value))
        }
    }
}
impl<'de> ::serde::Deserialize<'de> for TerminalOpenedMessageProtocolVersion {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        Self::try_from(<i64>::deserialize(deserializer)?)
            .map_err(|e| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[doc = "`TerminalOpenedMessageType`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"terminal.opened\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum TerminalOpenedMessageType {
    #[serde(rename = "terminal.opened")]
    TerminalOpened,
}
impl ::std::fmt::Display for TerminalOpenedMessageType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::TerminalOpened => f.write_str("terminal.opened"),
        }
    }
}
impl ::std::str::FromStr for TerminalOpenedMessageType {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "terminal.opened" => Ok(Self::TerminalOpened),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for TerminalOpenedMessageType {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TerminalOpenedMessageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TerminalOpenedMessageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`TerminalResizeMessage`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"TerminalResizeMessage\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"deviceId\","]
#[doc = "    \"payload\","]
#[doc = "    \"protocolVersion\","]
#[doc = "    \"requestId\","]
#[doc = "    \"sessionId\","]
#[doc = "    \"type\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"deviceId\": {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    },"]
#[doc = "    \"payload\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"cols\","]
#[doc = "        \"rows\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"cols\": {"]
#[doc = "          \"$ref\": \"#/$defs/Cols\""]
#[doc = "        },"]
#[doc = "        \"rows\": {"]
#[doc = "          \"$ref\": \"#/$defs/Rows\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"protocolVersion\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"enum\": ["]
#[doc = "        1"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"requestId\": {"]
#[doc = "      \"type\": \"null\""]
#[doc = "    },"]
#[doc = "    \"sessionId\": {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    },"]
#[doc = "    \"type\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"terminal.resize\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TerminalResizeMessage {
    #[serde(rename = "deviceId")]
    pub device_id: Uuid,
    pub payload: TerminalResizeMessagePayload,
    #[serde(rename = "protocolVersion")]
    pub protocol_version: TerminalResizeMessageProtocolVersion,
    #[serde(rename = "requestId")]
    pub request_id: (),
    #[serde(rename = "sessionId")]
    pub session_id: Uuid,
    #[serde(rename = "type")]
    pub type_: TerminalResizeMessageType,
}
impl TerminalResizeMessage {
    pub fn builder() -> builder::TerminalResizeMessage {
        Default::default()
    }
}
#[doc = "`TerminalResizeMessagePayload`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"cols\","]
#[doc = "    \"rows\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"cols\": {"]
#[doc = "      \"$ref\": \"#/$defs/Cols\""]
#[doc = "    },"]
#[doc = "    \"rows\": {"]
#[doc = "      \"$ref\": \"#/$defs/Rows\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TerminalResizeMessagePayload {
    pub cols: Cols,
    pub rows: Rows,
}
impl TerminalResizeMessagePayload {
    pub fn builder() -> builder::TerminalResizeMessagePayload {
        Default::default()
    }
}
#[doc = "`TerminalResizeMessageProtocolVersion`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"integer\","]
#[doc = "  \"enum\": ["]
#[doc = "    1"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct TerminalResizeMessageProtocolVersion(i64);
impl ::std::ops::Deref for TerminalResizeMessageProtocolVersion {
    type Target = i64;
    fn deref(&self) -> &i64 {
        &self.0
    }
}
impl ::std::convert::From<TerminalResizeMessageProtocolVersion> for i64 {
    fn from(value: TerminalResizeMessageProtocolVersion) -> Self {
        value.0
    }
}
impl ::std::convert::TryFrom<i64> for TerminalResizeMessageProtocolVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: i64) -> ::std::result::Result<Self, self::error::ConversionError> {
        if ![1_i64].contains(&value) {
            Err("invalid value".into())
        } else {
            Ok(Self(value))
        }
    }
}
impl<'de> ::serde::Deserialize<'de> for TerminalResizeMessageProtocolVersion {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        Self::try_from(<i64>::deserialize(deserializer)?)
            .map_err(|e| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[doc = "`TerminalResizeMessageType`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"terminal.resize\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum TerminalResizeMessageType {
    #[serde(rename = "terminal.resize")]
    TerminalResize,
}
impl ::std::fmt::Display for TerminalResizeMessageType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::TerminalResize => f.write_str("terminal.resize"),
        }
    }
}
impl ::std::str::FromStr for TerminalResizeMessageType {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "terminal.resize" => Ok(Self::TerminalResize),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for TerminalResizeMessageType {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TerminalResizeMessageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TerminalResizeMessageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "Shell integration events produced by the Agent's port of rust-servers/src/pty/osc_scanner.rs. The payload is shape-identical to Termy's local ShellEvent so the plugin can forward it to the existing handler unchanged. cwd is deliberately absent: the plugin derives it by parsing the xterm buffer (extractCwdFromPromptLines), and the terminal output bytes that parsing needs are forwarded verbatim, so cwd tracking already works in remote mode without protocol support."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"TerminalShellEventMessage\","]
#[doc = "  \"description\": \"Shell integration events produced by the Agent's port of rust-servers/src/pty/osc_scanner.rs. The payload is shape-identical to Termy's local ShellEvent so the plugin can forward it to the existing handler unchanged. cwd is deliberately absent: the plugin derives it by parsing the xterm buffer (extractCwdFromPromptLines), and the terminal output bytes that parsing needs are forwarded verbatim, so cwd tracking already works in remote mode without protocol support.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"deviceId\","]
#[doc = "    \"payload\","]
#[doc = "    \"protocolVersion\","]
#[doc = "    \"requestId\","]
#[doc = "    \"sessionId\","]
#[doc = "    \"type\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"deviceId\": {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    },"]
#[doc = "    \"payload\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"exitCode\","]
#[doc = "        \"source\","]
#[doc = "        \"type\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"exitCode\": {"]
#[doc = "          \"oneOf\": ["]
#[doc = "            {"]
#[doc = "              \"type\": \"integer\","]
#[doc = "              \"maximum\": 2147483647.0,"]
#[doc = "              \"minimum\": -2147483648.0"]
#[doc = "            },"]
#[doc = "            {"]
#[doc = "              \"type\": \"null\""]
#[doc = "            }"]
#[doc = "          ]"]
#[doc = "        },"]
#[doc = "        \"source\": {"]
#[doc = "          \"$ref\": \"#/$defs/ShellEventSource\""]
#[doc = "        },"]
#[doc = "        \"type\": {"]
#[doc = "          \"$ref\": \"#/$defs/ShellEventName\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"protocolVersion\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"enum\": ["]
#[doc = "        1"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"requestId\": {"]
#[doc = "      \"type\": \"null\""]
#[doc = "    },"]
#[doc = "    \"sessionId\": {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    },"]
#[doc = "    \"type\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"terminal.shellEvent\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TerminalShellEventMessage {
    #[serde(rename = "deviceId")]
    pub device_id: Uuid,
    pub payload: TerminalShellEventMessagePayload,
    #[serde(rename = "protocolVersion")]
    pub protocol_version: TerminalShellEventMessageProtocolVersion,
    #[serde(rename = "requestId")]
    pub request_id: (),
    #[serde(rename = "sessionId")]
    pub session_id: Uuid,
    #[serde(rename = "type")]
    pub type_: TerminalShellEventMessageType,
}
impl TerminalShellEventMessage {
    pub fn builder() -> builder::TerminalShellEventMessage {
        Default::default()
    }
}
#[doc = "`TerminalShellEventMessagePayload`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"exitCode\","]
#[doc = "    \"source\","]
#[doc = "    \"type\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"exitCode\": {"]
#[doc = "      \"oneOf\": ["]
#[doc = "        {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"maximum\": 2147483647.0,"]
#[doc = "          \"minimum\": -2147483648.0"]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"source\": {"]
#[doc = "      \"$ref\": \"#/$defs/ShellEventSource\""]
#[doc = "    },"]
#[doc = "    \"type\": {"]
#[doc = "      \"$ref\": \"#/$defs/ShellEventName\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TerminalShellEventMessagePayload {
    #[serde(rename = "exitCode")]
    pub exit_code: ::std::option::Option<i32>,
    pub source: ShellEventSource,
    #[serde(rename = "type")]
    pub type_: ShellEventName,
}
impl TerminalShellEventMessagePayload {
    pub fn builder() -> builder::TerminalShellEventMessagePayload {
        Default::default()
    }
}
#[doc = "`TerminalShellEventMessageProtocolVersion`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"integer\","]
#[doc = "  \"enum\": ["]
#[doc = "    1"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct TerminalShellEventMessageProtocolVersion(i64);
impl ::std::ops::Deref for TerminalShellEventMessageProtocolVersion {
    type Target = i64;
    fn deref(&self) -> &i64 {
        &self.0
    }
}
impl ::std::convert::From<TerminalShellEventMessageProtocolVersion> for i64 {
    fn from(value: TerminalShellEventMessageProtocolVersion) -> Self {
        value.0
    }
}
impl ::std::convert::TryFrom<i64> for TerminalShellEventMessageProtocolVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: i64) -> ::std::result::Result<Self, self::error::ConversionError> {
        if ![1_i64].contains(&value) {
            Err("invalid value".into())
        } else {
            Ok(Self(value))
        }
    }
}
impl<'de> ::serde::Deserialize<'de> for TerminalShellEventMessageProtocolVersion {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        Self::try_from(<i64>::deserialize(deserializer)?)
            .map_err(|e| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[doc = "`TerminalShellEventMessageType`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"terminal.shellEvent\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum TerminalShellEventMessageType {
    #[serde(rename = "terminal.shellEvent")]
    TerminalShellEvent,
}
impl ::std::fmt::Display for TerminalShellEventMessageType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::TerminalShellEvent => f.write_str("terminal.shellEvent"),
        }
    }
}
impl ::std::str::FromStr for TerminalShellEventMessageType {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "terminal.shellEvent" => Ok(Self::TerminalShellEvent),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for TerminalShellEventMessageType {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TerminalShellEventMessageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TerminalShellEventMessageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "Plugin-side abort: read failure, user cancel, or leaving remote mode. The Agent stops accepting frames for this transferId, closes the open file handle and replies transfer.result with success=false. Partial files may remain."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"TransferAbortMessage\","]
#[doc = "  \"description\": \"Plugin-side abort: read failure, user cancel, or leaving remote mode. The Agent stops accepting frames for this transferId, closes the open file handle and replies transfer.result with success=false. Partial files may remain.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"deviceId\","]
#[doc = "    \"payload\","]
#[doc = "    \"protocolVersion\","]
#[doc = "    \"requestId\","]
#[doc = "    \"sessionId\","]
#[doc = "    \"type\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"deviceId\": {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    },"]
#[doc = "    \"payload\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"code\","]
#[doc = "        \"transferId\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"code\": {"]
#[doc = "          \"$ref\": \"#/$defs/ErrorCode\""]
#[doc = "        },"]
#[doc = "        \"transferId\": {"]
#[doc = "          \"$ref\": \"#/$defs/Uuid\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"protocolVersion\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"enum\": ["]
#[doc = "        1"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"requestId\": {"]
#[doc = "      \"type\": \"null\""]
#[doc = "    },"]
#[doc = "    \"sessionId\": {"]
#[doc = "      \"type\": \"null\""]
#[doc = "    },"]
#[doc = "    \"type\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"transfer.abort\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TransferAbortMessage {
    #[serde(rename = "deviceId")]
    pub device_id: Uuid,
    pub payload: TransferAbortMessagePayload,
    #[serde(rename = "protocolVersion")]
    pub protocol_version: TransferAbortMessageProtocolVersion,
    #[serde(rename = "requestId")]
    pub request_id: (),
    #[serde(rename = "sessionId")]
    pub session_id: (),
    #[serde(rename = "type")]
    pub type_: TransferAbortMessageType,
}
impl TransferAbortMessage {
    pub fn builder() -> builder::TransferAbortMessage {
        Default::default()
    }
}
#[doc = "`TransferAbortMessagePayload`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"code\","]
#[doc = "    \"transferId\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"code\": {"]
#[doc = "      \"$ref\": \"#/$defs/ErrorCode\""]
#[doc = "    },"]
#[doc = "    \"transferId\": {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TransferAbortMessagePayload {
    pub code: ErrorCode,
    #[serde(rename = "transferId")]
    pub transfer_id: Uuid,
}
impl TransferAbortMessagePayload {
    pub fn builder() -> builder::TransferAbortMessagePayload {
        Default::default()
    }
}
#[doc = "`TransferAbortMessageProtocolVersion`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"integer\","]
#[doc = "  \"enum\": ["]
#[doc = "    1"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct TransferAbortMessageProtocolVersion(i64);
impl ::std::ops::Deref for TransferAbortMessageProtocolVersion {
    type Target = i64;
    fn deref(&self) -> &i64 {
        &self.0
    }
}
impl ::std::convert::From<TransferAbortMessageProtocolVersion> for i64 {
    fn from(value: TransferAbortMessageProtocolVersion) -> Self {
        value.0
    }
}
impl ::std::convert::TryFrom<i64> for TransferAbortMessageProtocolVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: i64) -> ::std::result::Result<Self, self::error::ConversionError> {
        if ![1_i64].contains(&value) {
            Err("invalid value".into())
        } else {
            Ok(Self(value))
        }
    }
}
impl<'de> ::serde::Deserialize<'de> for TransferAbortMessageProtocolVersion {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        Self::try_from(<i64>::deserialize(deserializer)?)
            .map_err(|e| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[doc = "`TransferAbortMessageType`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"transfer.abort\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum TransferAbortMessageType {
    #[serde(rename = "transfer.abort")]
    TransferAbort,
}
impl ::std::fmt::Display for TransferAbortMessageType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::TransferAbort => f.write_str("transfer.abort"),
        }
    }
}
impl ::std::str::FromStr for TransferAbortMessageType {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "transfer.abort" => Ok(Self::TransferAbort),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for TransferAbortMessageType {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TransferAbortMessageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TransferAbortMessageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "Direct response to transfer.start. grantedBytes is the initial credit window (doc 8.6): the plugin may not send file frames before this message, and never more than grantedBytes cumulative bytes."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"TransferAcceptedMessage\","]
#[doc = "  \"description\": \"Direct response to transfer.start. grantedBytes is the initial credit window (doc 8.6): the plugin may not send file frames before this message, and never more than grantedBytes cumulative bytes.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"deviceId\","]
#[doc = "    \"payload\","]
#[doc = "    \"protocolVersion\","]
#[doc = "    \"requestId\","]
#[doc = "    \"sessionId\","]
#[doc = "    \"type\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"deviceId\": {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    },"]
#[doc = "    \"payload\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"grantedBytes\","]
#[doc = "        \"transferId\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"grantedBytes\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"maximum\": 4194304.0,"]
#[doc = "          \"minimum\": 262144.0"]
#[doc = "        },"]
#[doc = "        \"transferId\": {"]
#[doc = "          \"$ref\": \"#/$defs/Uuid\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"protocolVersion\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"enum\": ["]
#[doc = "        1"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"requestId\": {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    },"]
#[doc = "    \"sessionId\": {"]
#[doc = "      \"type\": \"null\""]
#[doc = "    },"]
#[doc = "    \"type\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"transfer.accepted\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TransferAcceptedMessage {
    #[serde(rename = "deviceId")]
    pub device_id: Uuid,
    pub payload: TransferAcceptedMessagePayload,
    #[serde(rename = "protocolVersion")]
    pub protocol_version: TransferAcceptedMessageProtocolVersion,
    #[serde(rename = "requestId")]
    pub request_id: Uuid,
    #[serde(rename = "sessionId")]
    pub session_id: (),
    #[serde(rename = "type")]
    pub type_: TransferAcceptedMessageType,
}
impl TransferAcceptedMessage {
    pub fn builder() -> builder::TransferAcceptedMessage {
        Default::default()
    }
}
#[doc = "`TransferAcceptedMessagePayload`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"grantedBytes\","]
#[doc = "    \"transferId\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"grantedBytes\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 4194304.0,"]
#[doc = "      \"minimum\": 262144.0"]
#[doc = "    },"]
#[doc = "    \"transferId\": {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TransferAcceptedMessagePayload {
    #[serde(rename = "grantedBytes")]
    pub granted_bytes: i64,
    #[serde(rename = "transferId")]
    pub transfer_id: Uuid,
}
impl TransferAcceptedMessagePayload {
    pub fn builder() -> builder::TransferAcceptedMessagePayload {
        Default::default()
    }
}
#[doc = "`TransferAcceptedMessageProtocolVersion`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"integer\","]
#[doc = "  \"enum\": ["]
#[doc = "    1"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct TransferAcceptedMessageProtocolVersion(i64);
impl ::std::ops::Deref for TransferAcceptedMessageProtocolVersion {
    type Target = i64;
    fn deref(&self) -> &i64 {
        &self.0
    }
}
impl ::std::convert::From<TransferAcceptedMessageProtocolVersion> for i64 {
    fn from(value: TransferAcceptedMessageProtocolVersion) -> Self {
        value.0
    }
}
impl ::std::convert::TryFrom<i64> for TransferAcceptedMessageProtocolVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: i64) -> ::std::result::Result<Self, self::error::ConversionError> {
        if ![1_i64].contains(&value) {
            Err("invalid value".into())
        } else {
            Ok(Self(value))
        }
    }
}
impl<'de> ::serde::Deserialize<'de> for TransferAcceptedMessageProtocolVersion {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        Self::try_from(<i64>::deserialize(deserializer)?)
            .map_err(|e| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[doc = "`TransferAcceptedMessageType`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"transfer.accepted\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum TransferAcceptedMessageType {
    #[serde(rename = "transfer.accepted")]
    TransferAccepted,
}
impl ::std::fmt::Display for TransferAcceptedMessageType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::TransferAccepted => f.write_str("transfer.accepted"),
        }
    }
}
impl ::std::str::FromStr for TransferAcceptedMessageType {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "transfer.accepted" => Ok(Self::TransferAccepted),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for TransferAcceptedMessageType {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TransferAcceptedMessageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TransferAcceptedMessageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "`TransferCompleteMessage`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"TransferCompleteMessage\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"deviceId\","]
#[doc = "    \"payload\","]
#[doc = "    \"protocolVersion\","]
#[doc = "    \"requestId\","]
#[doc = "    \"sessionId\","]
#[doc = "    \"type\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"deviceId\": {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    },"]
#[doc = "    \"payload\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"transferId\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"transferId\": {"]
#[doc = "          \"$ref\": \"#/$defs/Uuid\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"protocolVersion\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"enum\": ["]
#[doc = "        1"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"requestId\": {"]
#[doc = "      \"type\": \"null\""]
#[doc = "    },"]
#[doc = "    \"sessionId\": {"]
#[doc = "      \"type\": \"null\""]
#[doc = "    },"]
#[doc = "    \"type\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"transfer.complete\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TransferCompleteMessage {
    #[serde(rename = "deviceId")]
    pub device_id: Uuid,
    pub payload: TransferCompleteMessagePayload,
    #[serde(rename = "protocolVersion")]
    pub protocol_version: TransferCompleteMessageProtocolVersion,
    #[serde(rename = "requestId")]
    pub request_id: (),
    #[serde(rename = "sessionId")]
    pub session_id: (),
    #[serde(rename = "type")]
    pub type_: TransferCompleteMessageType,
}
impl TransferCompleteMessage {
    pub fn builder() -> builder::TransferCompleteMessage {
        Default::default()
    }
}
#[doc = "`TransferCompleteMessagePayload`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"transferId\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"transferId\": {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TransferCompleteMessagePayload {
    #[serde(rename = "transferId")]
    pub transfer_id: Uuid,
}
impl TransferCompleteMessagePayload {
    pub fn builder() -> builder::TransferCompleteMessagePayload {
        Default::default()
    }
}
#[doc = "`TransferCompleteMessageProtocolVersion`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"integer\","]
#[doc = "  \"enum\": ["]
#[doc = "    1"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct TransferCompleteMessageProtocolVersion(i64);
impl ::std::ops::Deref for TransferCompleteMessageProtocolVersion {
    type Target = i64;
    fn deref(&self) -> &i64 {
        &self.0
    }
}
impl ::std::convert::From<TransferCompleteMessageProtocolVersion> for i64 {
    fn from(value: TransferCompleteMessageProtocolVersion) -> Self {
        value.0
    }
}
impl ::std::convert::TryFrom<i64> for TransferCompleteMessageProtocolVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: i64) -> ::std::result::Result<Self, self::error::ConversionError> {
        if ![1_i64].contains(&value) {
            Err("invalid value".into())
        } else {
            Ok(Self(value))
        }
    }
}
impl<'de> ::serde::Deserialize<'de> for TransferCompleteMessageProtocolVersion {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        Self::try_from(<i64>::deserialize(deserializer)?)
            .map_err(|e| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[doc = "`TransferCompleteMessageType`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"transfer.complete\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum TransferCompleteMessageType {
    #[serde(rename = "transfer.complete")]
    TransferComplete,
}
impl ::std::fmt::Display for TransferCompleteMessageType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::TransferComplete => f.write_str("transfer.complete"),
        }
    }
}
impl ::std::str::FromStr for TransferCompleteMessageType {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "transfer.complete" => Ok(Self::TransferComplete),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for TransferCompleteMessageType {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TransferCompleteMessageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TransferCompleteMessageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "Credit top-up, sent after every 1 MiB flushed to disk. grantedBytes is the CUMULATIVE authorisation for the whole transfer and must increase monotonically; the receiver keeps the maximum it has seen. Upper bound is the 256 MiB per-transfer cap."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"TransferCreditMessage\","]
#[doc = "  \"description\": \"Credit top-up, sent after every 1 MiB flushed to disk. grantedBytes is the CUMULATIVE authorisation for the whole transfer and must increase monotonically; the receiver keeps the maximum it has seen. Upper bound is the 256 MiB per-transfer cap.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"deviceId\","]
#[doc = "    \"payload\","]
#[doc = "    \"protocolVersion\","]
#[doc = "    \"requestId\","]
#[doc = "    \"sessionId\","]
#[doc = "    \"type\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"deviceId\": {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    },"]
#[doc = "    \"payload\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"grantedBytes\","]
#[doc = "        \"transferId\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"grantedBytes\": {"]
#[doc = "          \"type\": \"integer\","]
#[doc = "          \"maximum\": 268435456.0,"]
#[doc = "          \"minimum\": 262144.0"]
#[doc = "        },"]
#[doc = "        \"transferId\": {"]
#[doc = "          \"$ref\": \"#/$defs/Uuid\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"protocolVersion\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"enum\": ["]
#[doc = "        1"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"requestId\": {"]
#[doc = "      \"type\": \"null\""]
#[doc = "    },"]
#[doc = "    \"sessionId\": {"]
#[doc = "      \"type\": \"null\""]
#[doc = "    },"]
#[doc = "    \"type\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"transfer.credit\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TransferCreditMessage {
    #[serde(rename = "deviceId")]
    pub device_id: Uuid,
    pub payload: TransferCreditMessagePayload,
    #[serde(rename = "protocolVersion")]
    pub protocol_version: TransferCreditMessageProtocolVersion,
    #[serde(rename = "requestId")]
    pub request_id: (),
    #[serde(rename = "sessionId")]
    pub session_id: (),
    #[serde(rename = "type")]
    pub type_: TransferCreditMessageType,
}
impl TransferCreditMessage {
    pub fn builder() -> builder::TransferCreditMessage {
        Default::default()
    }
}
#[doc = "`TransferCreditMessagePayload`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"grantedBytes\","]
#[doc = "    \"transferId\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"grantedBytes\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"maximum\": 268435456.0,"]
#[doc = "      \"minimum\": 262144.0"]
#[doc = "    },"]
#[doc = "    \"transferId\": {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TransferCreditMessagePayload {
    #[serde(rename = "grantedBytes")]
    pub granted_bytes: i64,
    #[serde(rename = "transferId")]
    pub transfer_id: Uuid,
}
impl TransferCreditMessagePayload {
    pub fn builder() -> builder::TransferCreditMessagePayload {
        Default::default()
    }
}
#[doc = "`TransferCreditMessageProtocolVersion`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"integer\","]
#[doc = "  \"enum\": ["]
#[doc = "    1"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct TransferCreditMessageProtocolVersion(i64);
impl ::std::ops::Deref for TransferCreditMessageProtocolVersion {
    type Target = i64;
    fn deref(&self) -> &i64 {
        &self.0
    }
}
impl ::std::convert::From<TransferCreditMessageProtocolVersion> for i64 {
    fn from(value: TransferCreditMessageProtocolVersion) -> Self {
        value.0
    }
}
impl ::std::convert::TryFrom<i64> for TransferCreditMessageProtocolVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: i64) -> ::std::result::Result<Self, self::error::ConversionError> {
        if ![1_i64].contains(&value) {
            Err("invalid value".into())
        } else {
            Ok(Self(value))
        }
    }
}
impl<'de> ::serde::Deserialize<'de> for TransferCreditMessageProtocolVersion {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        Self::try_from(<i64>::deserialize(deserializer)?)
            .map_err(|e| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[doc = "`TransferCreditMessageType`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"transfer.credit\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum TransferCreditMessageType {
    #[serde(rename = "transfer.credit")]
    TransferCredit,
}
impl ::std::fmt::Display for TransferCreditMessageType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::TransferCredit => f.write_str("transfer.credit"),
        }
    }
}
impl ::std::str::FromStr for TransferCreditMessageType {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "transfer.credit" => Ok(Self::TransferCredit),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for TransferCreditMessageType {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TransferCreditMessageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TransferCreditMessageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "Marks the end of one file. sentSize is authoritative for the success check (doc 10.4); a zero value means an empty file, for which no chunk frame is sent and the Agent must still create and close a 0-byte file."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"TransferFileEndMessage\","]
#[doc = "  \"description\": \"Marks the end of one file. sentSize is authoritative for the success check (doc 10.4); a zero value means an empty file, for which no chunk frame is sent and the Agent must still create and close a 0-byte file.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"deviceId\","]
#[doc = "    \"payload\","]
#[doc = "    \"protocolVersion\","]
#[doc = "    \"requestId\","]
#[doc = "    \"sessionId\","]
#[doc = "    \"type\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"deviceId\": {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    },"]
#[doc = "    \"payload\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"fileIndex\","]
#[doc = "        \"sentSize\","]
#[doc = "        \"transferId\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"fileIndex\": {"]
#[doc = "          \"$ref\": \"#/$defs/FileIndex\""]
#[doc = "        },"]
#[doc = "        \"sentSize\": {"]
#[doc = "          \"$ref\": \"#/$defs/FileSize\""]
#[doc = "        },"]
#[doc = "        \"transferId\": {"]
#[doc = "          \"$ref\": \"#/$defs/Uuid\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"protocolVersion\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"enum\": ["]
#[doc = "        1"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"requestId\": {"]
#[doc = "      \"type\": \"null\""]
#[doc = "    },"]
#[doc = "    \"sessionId\": {"]
#[doc = "      \"type\": \"null\""]
#[doc = "    },"]
#[doc = "    \"type\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"transfer.fileEnd\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TransferFileEndMessage {
    #[serde(rename = "deviceId")]
    pub device_id: Uuid,
    pub payload: TransferFileEndMessagePayload,
    #[serde(rename = "protocolVersion")]
    pub protocol_version: TransferFileEndMessageProtocolVersion,
    #[serde(rename = "requestId")]
    pub request_id: (),
    #[serde(rename = "sessionId")]
    pub session_id: (),
    #[serde(rename = "type")]
    pub type_: TransferFileEndMessageType,
}
impl TransferFileEndMessage {
    pub fn builder() -> builder::TransferFileEndMessage {
        Default::default()
    }
}
#[doc = "`TransferFileEndMessagePayload`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"fileIndex\","]
#[doc = "    \"sentSize\","]
#[doc = "    \"transferId\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"fileIndex\": {"]
#[doc = "      \"$ref\": \"#/$defs/FileIndex\""]
#[doc = "    },"]
#[doc = "    \"sentSize\": {"]
#[doc = "      \"$ref\": \"#/$defs/FileSize\""]
#[doc = "    },"]
#[doc = "    \"transferId\": {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TransferFileEndMessagePayload {
    #[serde(rename = "fileIndex")]
    pub file_index: FileIndex,
    #[serde(rename = "sentSize")]
    pub sent_size: FileSize,
    #[serde(rename = "transferId")]
    pub transfer_id: Uuid,
}
impl TransferFileEndMessagePayload {
    pub fn builder() -> builder::TransferFileEndMessagePayload {
        Default::default()
    }
}
#[doc = "`TransferFileEndMessageProtocolVersion`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"integer\","]
#[doc = "  \"enum\": ["]
#[doc = "    1"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct TransferFileEndMessageProtocolVersion(i64);
impl ::std::ops::Deref for TransferFileEndMessageProtocolVersion {
    type Target = i64;
    fn deref(&self) -> &i64 {
        &self.0
    }
}
impl ::std::convert::From<TransferFileEndMessageProtocolVersion> for i64 {
    fn from(value: TransferFileEndMessageProtocolVersion) -> Self {
        value.0
    }
}
impl ::std::convert::TryFrom<i64> for TransferFileEndMessageProtocolVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: i64) -> ::std::result::Result<Self, self::error::ConversionError> {
        if ![1_i64].contains(&value) {
            Err("invalid value".into())
        } else {
            Ok(Self(value))
        }
    }
}
impl<'de> ::serde::Deserialize<'de> for TransferFileEndMessageProtocolVersion {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        Self::try_from(<i64>::deserialize(deserializer)?)
            .map_err(|e| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[doc = "`TransferFileEndMessageType`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"transfer.fileEnd\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum TransferFileEndMessageType {
    #[serde(rename = "transfer.fileEnd")]
    TransferFileEnd,
}
impl ::std::fmt::Display for TransferFileEndMessageType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::TransferFileEnd => f.write_str("transfer.fileEnd"),
        }
    }
}
impl ::std::str::FromStr for TransferFileEndMessageType {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "transfer.fileEnd" => Ok(Self::TransferFileEnd),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for TransferFileEndMessageType {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TransferFileEndMessageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TransferFileEndMessageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "Terminal outcome of one transfer. Not a direct response to transfer.start (that is transfer.accepted), so requestId is null and correlation is by transferId. code is null exactly when success is true - a cross-field rule enforced in code."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"TransferResultMessage\","]
#[doc = "  \"description\": \"Terminal outcome of one transfer. Not a direct response to transfer.start (that is transfer.accepted), so requestId is null and correlation is by transferId. code is null exactly when success is true - a cross-field rule enforced in code.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"deviceId\","]
#[doc = "    \"payload\","]
#[doc = "    \"protocolVersion\","]
#[doc = "    \"requestId\","]
#[doc = "    \"sessionId\","]
#[doc = "    \"type\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"deviceId\": {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    },"]
#[doc = "    \"payload\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"code\","]
#[doc = "        \"message\","]
#[doc = "        \"success\","]
#[doc = "        \"transferId\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"code\": {"]
#[doc = "          \"oneOf\": ["]
#[doc = "            {"]
#[doc = "              \"$ref\": \"#/$defs/ErrorCode\""]
#[doc = "            },"]
#[doc = "            {"]
#[doc = "              \"type\": \"null\""]
#[doc = "            }"]
#[doc = "          ]"]
#[doc = "        },"]
#[doc = "        \"message\": {"]
#[doc = "          \"type\": \"string\","]
#[doc = "          \"maxLength\": 512"]
#[doc = "        },"]
#[doc = "        \"success\": {"]
#[doc = "          \"type\": \"boolean\""]
#[doc = "        },"]
#[doc = "        \"transferId\": {"]
#[doc = "          \"$ref\": \"#/$defs/Uuid\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"protocolVersion\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"enum\": ["]
#[doc = "        1"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"requestId\": {"]
#[doc = "      \"type\": \"null\""]
#[doc = "    },"]
#[doc = "    \"sessionId\": {"]
#[doc = "      \"type\": \"null\""]
#[doc = "    },"]
#[doc = "    \"type\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"transfer.result\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TransferResultMessage {
    #[serde(rename = "deviceId")]
    pub device_id: Uuid,
    pub payload: TransferResultMessagePayload,
    #[serde(rename = "protocolVersion")]
    pub protocol_version: TransferResultMessageProtocolVersion,
    #[serde(rename = "requestId")]
    pub request_id: (),
    #[serde(rename = "sessionId")]
    pub session_id: (),
    #[serde(rename = "type")]
    pub type_: TransferResultMessageType,
}
impl TransferResultMessage {
    pub fn builder() -> builder::TransferResultMessage {
        Default::default()
    }
}
#[doc = "`TransferResultMessagePayload`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"code\","]
#[doc = "    \"message\","]
#[doc = "    \"success\","]
#[doc = "    \"transferId\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"code\": {"]
#[doc = "      \"oneOf\": ["]
#[doc = "        {"]
#[doc = "          \"$ref\": \"#/$defs/ErrorCode\""]
#[doc = "        },"]
#[doc = "        {"]
#[doc = "          \"type\": \"null\""]
#[doc = "        }"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"message\": {"]
#[doc = "      \"type\": \"string\","]
#[doc = "      \"maxLength\": 512"]
#[doc = "    },"]
#[doc = "    \"success\": {"]
#[doc = "      \"type\": \"boolean\""]
#[doc = "    },"]
#[doc = "    \"transferId\": {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TransferResultMessagePayload {
    pub code: ::std::option::Option<ErrorCode>,
    pub message: TransferResultMessagePayloadMessage,
    pub success: bool,
    #[serde(rename = "transferId")]
    pub transfer_id: Uuid,
}
impl TransferResultMessagePayload {
    pub fn builder() -> builder::TransferResultMessagePayload {
        Default::default()
    }
}
#[doc = "`TransferResultMessagePayloadMessage`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"maxLength\": 512"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct TransferResultMessagePayloadMessage(::std::string::String);
impl ::std::ops::Deref for TransferResultMessagePayloadMessage {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<TransferResultMessagePayloadMessage> for ::std::string::String {
    fn from(value: TransferResultMessagePayloadMessage) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for TransferResultMessagePayloadMessage {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        if value.chars().count() > 512usize {
            return Err("longer than 512 characters".into());
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for TransferResultMessagePayloadMessage {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TransferResultMessagePayloadMessage {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TransferResultMessagePayloadMessage {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for TransferResultMessagePayloadMessage {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
#[doc = "`TransferResultMessageProtocolVersion`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"integer\","]
#[doc = "  \"enum\": ["]
#[doc = "    1"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct TransferResultMessageProtocolVersion(i64);
impl ::std::ops::Deref for TransferResultMessageProtocolVersion {
    type Target = i64;
    fn deref(&self) -> &i64 {
        &self.0
    }
}
impl ::std::convert::From<TransferResultMessageProtocolVersion> for i64 {
    fn from(value: TransferResultMessageProtocolVersion) -> Self {
        value.0
    }
}
impl ::std::convert::TryFrom<i64> for TransferResultMessageProtocolVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: i64) -> ::std::result::Result<Self, self::error::ConversionError> {
        if ![1_i64].contains(&value) {
            Err("invalid value".into())
        } else {
            Ok(Self(value))
        }
    }
}
impl<'de> ::serde::Deserialize<'de> for TransferResultMessageProtocolVersion {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        Self::try_from(<i64>::deserialize(deserializer)?)
            .map_err(|e| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[doc = "`TransferResultMessageType`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"transfer.result\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum TransferResultMessageType {
    #[serde(rename = "transfer.result")]
    TransferResult,
}
impl ::std::fmt::Display for TransferResultMessageType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::TransferResult => f.write_str("transfer.result"),
        }
    }
}
impl ::std::str::FromStr for TransferResultMessageType {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "transfer.result" => Ok(Self::TransferResult),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for TransferResultMessageType {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TransferResultMessageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TransferResultMessageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "File transfer is independent of any terminal session, so sessionId is null. entries[].index must run 0..n-1 with no gaps and rootNote must equal entries[0].relativePath; both are cross-field rules enforced in code, not expressible here."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"TransferStartMessage\","]
#[doc = "  \"description\": \"File transfer is independent of any terminal session, so sessionId is null. entries[].index must run 0..n-1 with no gaps and rootNote must equal entries[0].relativePath; both are cross-field rules enforced in code, not expressible here.\","]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"deviceId\","]
#[doc = "    \"payload\","]
#[doc = "    \"protocolVersion\","]
#[doc = "    \"requestId\","]
#[doc = "    \"sessionId\","]
#[doc = "    \"type\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"deviceId\": {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    },"]
#[doc = "    \"payload\": {"]
#[doc = "      \"type\": \"object\","]
#[doc = "      \"required\": ["]
#[doc = "        \"entries\","]
#[doc = "        \"rootNote\","]
#[doc = "        \"transferId\""]
#[doc = "      ],"]
#[doc = "      \"properties\": {"]
#[doc = "        \"entries\": {"]
#[doc = "          \"type\": \"array\","]
#[doc = "          \"items\": {"]
#[doc = "            \"$ref\": \"#/$defs/FileEntry\""]
#[doc = "          },"]
#[doc = "          \"maxItems\": 256,"]
#[doc = "          \"minItems\": 1"]
#[doc = "        },"]
#[doc = "        \"rootNote\": {"]
#[doc = "          \"$ref\": \"#/$defs/SafeRelativePath\""]
#[doc = "        },"]
#[doc = "        \"transferId\": {"]
#[doc = "          \"$ref\": \"#/$defs/Uuid\""]
#[doc = "        }"]
#[doc = "      },"]
#[doc = "      \"additionalProperties\": false"]
#[doc = "    },"]
#[doc = "    \"protocolVersion\": {"]
#[doc = "      \"type\": \"integer\","]
#[doc = "      \"enum\": ["]
#[doc = "        1"]
#[doc = "      ]"]
#[doc = "    },"]
#[doc = "    \"requestId\": {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    },"]
#[doc = "    \"sessionId\": {"]
#[doc = "      \"type\": \"null\""]
#[doc = "    },"]
#[doc = "    \"type\": {"]
#[doc = "      \"enum\": ["]
#[doc = "        \"transfer.start\""]
#[doc = "      ]"]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TransferStartMessage {
    #[serde(rename = "deviceId")]
    pub device_id: Uuid,
    pub payload: TransferStartMessagePayload,
    #[serde(rename = "protocolVersion")]
    pub protocol_version: TransferStartMessageProtocolVersion,
    #[serde(rename = "requestId")]
    pub request_id: Uuid,
    #[serde(rename = "sessionId")]
    pub session_id: (),
    #[serde(rename = "type")]
    pub type_: TransferStartMessageType,
}
impl TransferStartMessage {
    pub fn builder() -> builder::TransferStartMessage {
        Default::default()
    }
}
#[doc = "`TransferStartMessagePayload`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"object\","]
#[doc = "  \"required\": ["]
#[doc = "    \"entries\","]
#[doc = "    \"rootNote\","]
#[doc = "    \"transferId\""]
#[doc = "  ],"]
#[doc = "  \"properties\": {"]
#[doc = "    \"entries\": {"]
#[doc = "      \"type\": \"array\","]
#[doc = "      \"items\": {"]
#[doc = "        \"$ref\": \"#/$defs/FileEntry\""]
#[doc = "      },"]
#[doc = "      \"maxItems\": 256,"]
#[doc = "      \"minItems\": 1"]
#[doc = "    },"]
#[doc = "    \"rootNote\": {"]
#[doc = "      \"$ref\": \"#/$defs/SafeRelativePath\""]
#[doc = "    },"]
#[doc = "    \"transferId\": {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    }"]
#[doc = "  },"]
#[doc = "  \"additionalProperties\": false"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TransferStartMessagePayload {
    pub entries: ::std::vec::Vec<FileEntry>,
    #[serde(rename = "rootNote")]
    pub root_note: SafeRelativePath,
    #[serde(rename = "transferId")]
    pub transfer_id: Uuid,
}
impl TransferStartMessagePayload {
    pub fn builder() -> builder::TransferStartMessagePayload {
        Default::default()
    }
}
#[doc = "`TransferStartMessageProtocolVersion`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"type\": \"integer\","]
#[doc = "  \"enum\": ["]
#[doc = "    1"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct TransferStartMessageProtocolVersion(i64);
impl ::std::ops::Deref for TransferStartMessageProtocolVersion {
    type Target = i64;
    fn deref(&self) -> &i64 {
        &self.0
    }
}
impl ::std::convert::From<TransferStartMessageProtocolVersion> for i64 {
    fn from(value: TransferStartMessageProtocolVersion) -> Self {
        value.0
    }
}
impl ::std::convert::TryFrom<i64> for TransferStartMessageProtocolVersion {
    type Error = self::error::ConversionError;
    fn try_from(value: i64) -> ::std::result::Result<Self, self::error::ConversionError> {
        if ![1_i64].contains(&value) {
            Err("invalid value".into())
        } else {
            Ok(Self(value))
        }
    }
}
impl<'de> ::serde::Deserialize<'de> for TransferStartMessageProtocolVersion {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        Self::try_from(<i64>::deserialize(deserializer)?)
            .map_err(|e| <D::Error as ::serde::de::Error>::custom(e.to_string()))
    }
}
#[doc = "`TransferStartMessageType`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"enum\": ["]
#[doc = "    \"transfer.start\""]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(
    :: serde :: Deserialize,
    :: serde :: Serialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum TransferStartMessageType {
    #[serde(rename = "transfer.start")]
    TransferStart,
}
impl ::std::fmt::Display for TransferStartMessageType {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match *self {
            Self::TransferStart => f.write_str("transfer.start"),
        }
    }
}
impl ::std::str::FromStr for TransferStartMessageType {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        match value {
            "transfer.start" => Ok(Self::TransferStart),
            _ => Err("invalid value".into()),
        }
    }
}
impl ::std::convert::TryFrom<&str> for TransferStartMessageType {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for TransferStartMessageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for TransferStartMessageType {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
#[doc = "Lowercase hyphenated UUID."]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"Uuid\","]
#[doc = "  \"description\": \"Lowercase hyphenated UUID.\","]
#[doc = "  \"type\": \"string\","]
#[doc = "  \"pattern\": \"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$\""]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Serialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct Uuid(::std::string::String);
impl ::std::ops::Deref for Uuid {
    type Target = ::std::string::String;
    fn deref(&self) -> &::std::string::String {
        &self.0
    }
}
impl ::std::convert::From<Uuid> for ::std::string::String {
    fn from(value: Uuid) -> Self {
        value.0
    }
}
impl ::std::str::FromStr for Uuid {
    type Err = self::error::ConversionError;
    fn from_str(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        static PATTERN: ::std::sync::LazyLock<::regress::Regex> =
            ::std::sync::LazyLock::new(|| {
                ::regress::Regex::new(
                    "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
                )
                .unwrap()
            });
        if PATTERN.find(value).is_none() {
            return Err ("doesn't match pattern \"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$\"" . into ()) ;
        }
        Ok(Self(value.to_string()))
    }
}
impl ::std::convert::TryFrom<&str> for Uuid {
    type Error = self::error::ConversionError;
    fn try_from(value: &str) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<&::std::string::String> for Uuid {
    type Error = self::error::ConversionError;
    fn try_from(
        value: &::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl ::std::convert::TryFrom<::std::string::String> for Uuid {
    type Error = self::error::ConversionError;
    fn try_from(
        value: ::std::string::String,
    ) -> ::std::result::Result<Self, self::error::ConversionError> {
        value.parse()
    }
}
impl<'de> ::serde::Deserialize<'de> for Uuid {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        ::std::string::String::deserialize(deserializer)?
            .parse()
            .map_err(|e: self::error::ConversionError| {
                <D::Error as ::serde::de::Error>::custom(e.to_string())
            })
    }
}
#[doc = "`UuidOrNull`"]
#[doc = r""]
#[doc = r" <details><summary>JSON schema</summary>"]
#[doc = r""]
#[doc = r" ```json"]
#[doc = "{"]
#[doc = "  \"title\": \"UuidOrNull\","]
#[doc = "  \"oneOf\": ["]
#[doc = "    {"]
#[doc = "      \"$ref\": \"#/$defs/Uuid\""]
#[doc = "    },"]
#[doc = "    {"]
#[doc = "      \"type\": \"null\""]
#[doc = "    }"]
#[doc = "  ]"]
#[doc = "}"]
#[doc = r" ```"]
#[doc = r" </details>"]
#[derive(:: serde :: Deserialize, :: serde :: Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct UuidOrNull(pub ::std::option::Option<Uuid>);
impl ::std::ops::Deref for UuidOrNull {
    type Target = ::std::option::Option<Uuid>;
    fn deref(&self) -> &::std::option::Option<Uuid> {
        &self.0
    }
}
impl ::std::convert::From<UuidOrNull> for ::std::option::Option<Uuid> {
    fn from(value: UuidOrNull) -> Self {
        value.0
    }
}
impl ::std::convert::From<::std::option::Option<Uuid>> for UuidOrNull {
    fn from(value: ::std::option::Option<Uuid>) -> Self {
        Self(value)
    }
}
#[doc = r" Types for composing complex structures."]
pub mod builder {
    #[derive(Clone, Debug)]
    pub struct AgentHeartbeatMessage {
        device_id: ::std::result::Result<super::Uuid, ::std::string::String>,
        payload: ::std::result::Result<super::AgentHeartbeatMessagePayload, ::std::string::String>,
        protocol_version: ::std::result::Result<
            super::AgentHeartbeatMessageProtocolVersion,
            ::std::string::String,
        >,
        request_id: ::std::result::Result<(), ::std::string::String>,
        session_id: ::std::result::Result<(), ::std::string::String>,
        type_: ::std::result::Result<super::AgentHeartbeatMessageType, ::std::string::String>,
    }
    impl ::std::default::Default for AgentHeartbeatMessage {
        fn default() -> Self {
            Self {
                device_id: Err("no value supplied for device_id".to_string()),
                payload: Err("no value supplied for payload".to_string()),
                protocol_version: Err("no value supplied for protocol_version".to_string()),
                request_id: Err("no value supplied for request_id".to_string()),
                session_id: Err("no value supplied for session_id".to_string()),
                type_: Err("no value supplied for type_".to_string()),
            }
        }
    }
    impl AgentHeartbeatMessage {
        pub fn device_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.device_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for device_id: {e}"));
            self
        }
        pub fn payload<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::AgentHeartbeatMessagePayload>,
            T::Error: ::std::fmt::Display,
        {
            self.payload = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for payload: {e}"));
            self
        }
        pub fn protocol_version<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::AgentHeartbeatMessageProtocolVersion>,
            T::Error: ::std::fmt::Display,
        {
            self.protocol_version = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for protocol_version: {e}"));
            self
        }
        pub fn request_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<()>,
            T::Error: ::std::fmt::Display,
        {
            self.request_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for request_id: {e}"));
            self
        }
        pub fn session_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<()>,
            T::Error: ::std::fmt::Display,
        {
            self.session_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for session_id: {e}"));
            self
        }
        pub fn type_<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::AgentHeartbeatMessageType>,
            T::Error: ::std::fmt::Display,
        {
            self.type_ = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for type_: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<AgentHeartbeatMessage> for super::AgentHeartbeatMessage {
        type Error = super::error::ConversionError;
        fn try_from(
            value: AgentHeartbeatMessage,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                device_id: value.device_id?,
                payload: value.payload?,
                protocol_version: value.protocol_version?,
                request_id: value.request_id?,
                session_id: value.session_id?,
                type_: value.type_?,
            })
        }
    }
    impl ::std::convert::From<super::AgentHeartbeatMessage> for AgentHeartbeatMessage {
        fn from(value: super::AgentHeartbeatMessage) -> Self {
            Self {
                device_id: Ok(value.device_id),
                payload: Ok(value.payload),
                protocol_version: Ok(value.protocol_version),
                request_id: Ok(value.request_id),
                session_id: Ok(value.session_id),
                type_: Ok(value.type_),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct AgentHeartbeatMessagePayload {
        timestamp: ::std::result::Result<super::DateTime, ::std::string::String>,
    }
    impl ::std::default::Default for AgentHeartbeatMessagePayload {
        fn default() -> Self {
            Self {
                timestamp: Err("no value supplied for timestamp".to_string()),
            }
        }
    }
    impl AgentHeartbeatMessagePayload {
        pub fn timestamp<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::DateTime>,
            T::Error: ::std::fmt::Display,
        {
            self.timestamp = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for timestamp: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<AgentHeartbeatMessagePayload> for super::AgentHeartbeatMessagePayload {
        type Error = super::error::ConversionError;
        fn try_from(
            value: AgentHeartbeatMessagePayload,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                timestamp: value.timestamp?,
            })
        }
    }
    impl ::std::convert::From<super::AgentHeartbeatMessagePayload> for AgentHeartbeatMessagePayload {
        fn from(value: super::AgentHeartbeatMessagePayload) -> Self {
            Self {
                timestamp: Ok(value.timestamp),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct AgentHelloAckMessage {
        device_id: ::std::result::Result<super::Uuid, ::std::string::String>,
        payload: ::std::result::Result<super::AgentHelloAckMessagePayload, ::std::string::String>,
        protocol_version: ::std::result::Result<
            super::AgentHelloAckMessageProtocolVersion,
            ::std::string::String,
        >,
        request_id: ::std::result::Result<super::Uuid, ::std::string::String>,
        session_id: ::std::result::Result<(), ::std::string::String>,
        type_: ::std::result::Result<super::AgentHelloAckMessageType, ::std::string::String>,
    }
    impl ::std::default::Default for AgentHelloAckMessage {
        fn default() -> Self {
            Self {
                device_id: Err("no value supplied for device_id".to_string()),
                payload: Err("no value supplied for payload".to_string()),
                protocol_version: Err("no value supplied for protocol_version".to_string()),
                request_id: Err("no value supplied for request_id".to_string()),
                session_id: Err("no value supplied for session_id".to_string()),
                type_: Err("no value supplied for type_".to_string()),
            }
        }
    }
    impl AgentHelloAckMessage {
        pub fn device_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.device_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for device_id: {e}"));
            self
        }
        pub fn payload<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::AgentHelloAckMessagePayload>,
            T::Error: ::std::fmt::Display,
        {
            self.payload = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for payload: {e}"));
            self
        }
        pub fn protocol_version<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::AgentHelloAckMessageProtocolVersion>,
            T::Error: ::std::fmt::Display,
        {
            self.protocol_version = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for protocol_version: {e}"));
            self
        }
        pub fn request_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.request_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for request_id: {e}"));
            self
        }
        pub fn session_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<()>,
            T::Error: ::std::fmt::Display,
        {
            self.session_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for session_id: {e}"));
            self
        }
        pub fn type_<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::AgentHelloAckMessageType>,
            T::Error: ::std::fmt::Display,
        {
            self.type_ = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for type_: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<AgentHelloAckMessage> for super::AgentHelloAckMessage {
        type Error = super::error::ConversionError;
        fn try_from(
            value: AgentHelloAckMessage,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                device_id: value.device_id?,
                payload: value.payload?,
                protocol_version: value.protocol_version?,
                request_id: value.request_id?,
                session_id: value.session_id?,
                type_: value.type_?,
            })
        }
    }
    impl ::std::convert::From<super::AgentHelloAckMessage> for AgentHelloAckMessage {
        fn from(value: super::AgentHelloAckMessage) -> Self {
            Self {
                device_id: Ok(value.device_id),
                payload: Ok(value.payload),
                protocol_version: Ok(value.protocol_version),
                request_id: Ok(value.request_id),
                session_id: Ok(value.session_id),
                type_: Ok(value.type_),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct AgentHelloAckMessagePayload {
        heartbeat_interval_ms: ::std::result::Result<
            super::AgentHelloAckMessagePayloadHeartbeatIntervalMs,
            ::std::string::String,
        >,
        server_time: ::std::result::Result<super::DateTime, ::std::string::String>,
    }
    impl ::std::default::Default for AgentHelloAckMessagePayload {
        fn default() -> Self {
            Self {
                heartbeat_interval_ms: Err(
                    "no value supplied for heartbeat_interval_ms".to_string()
                ),
                server_time: Err("no value supplied for server_time".to_string()),
            }
        }
    }
    impl AgentHelloAckMessagePayload {
        pub fn heartbeat_interval_ms<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::AgentHelloAckMessagePayloadHeartbeatIntervalMs>,
            T::Error: ::std::fmt::Display,
        {
            self.heartbeat_interval_ms = value.try_into().map_err(|e| {
                format!("error converting supplied value for heartbeat_interval_ms: {e}")
            });
            self
        }
        pub fn server_time<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::DateTime>,
            T::Error: ::std::fmt::Display,
        {
            self.server_time = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for server_time: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<AgentHelloAckMessagePayload> for super::AgentHelloAckMessagePayload {
        type Error = super::error::ConversionError;
        fn try_from(
            value: AgentHelloAckMessagePayload,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                heartbeat_interval_ms: value.heartbeat_interval_ms?,
                server_time: value.server_time?,
            })
        }
    }
    impl ::std::convert::From<super::AgentHelloAckMessagePayload> for AgentHelloAckMessagePayload {
        fn from(value: super::AgentHelloAckMessagePayload) -> Self {
            Self {
                heartbeat_interval_ms: Ok(value.heartbeat_interval_ms),
                server_time: Ok(value.server_time),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct AgentHelloMessage {
        device_id: ::std::result::Result<super::Uuid, ::std::string::String>,
        payload: ::std::result::Result<super::AgentHelloMessagePayload, ::std::string::String>,
        protocol_version:
            ::std::result::Result<super::AgentHelloMessageProtocolVersion, ::std::string::String>,
        request_id: ::std::result::Result<super::Uuid, ::std::string::String>,
        session_id: ::std::result::Result<(), ::std::string::String>,
        type_: ::std::result::Result<super::AgentHelloMessageType, ::std::string::String>,
    }
    impl ::std::default::Default for AgentHelloMessage {
        fn default() -> Self {
            Self {
                device_id: Err("no value supplied for device_id".to_string()),
                payload: Err("no value supplied for payload".to_string()),
                protocol_version: Err("no value supplied for protocol_version".to_string()),
                request_id: Err("no value supplied for request_id".to_string()),
                session_id: Err("no value supplied for session_id".to_string()),
                type_: Err("no value supplied for type_".to_string()),
            }
        }
    }
    impl AgentHelloMessage {
        pub fn device_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.device_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for device_id: {e}"));
            self
        }
        pub fn payload<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::AgentHelloMessagePayload>,
            T::Error: ::std::fmt::Display,
        {
            self.payload = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for payload: {e}"));
            self
        }
        pub fn protocol_version<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::AgentHelloMessageProtocolVersion>,
            T::Error: ::std::fmt::Display,
        {
            self.protocol_version = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for protocol_version: {e}"));
            self
        }
        pub fn request_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.request_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for request_id: {e}"));
            self
        }
        pub fn session_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<()>,
            T::Error: ::std::fmt::Display,
        {
            self.session_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for session_id: {e}"));
            self
        }
        pub fn type_<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::AgentHelloMessageType>,
            T::Error: ::std::fmt::Display,
        {
            self.type_ = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for type_: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<AgentHelloMessage> for super::AgentHelloMessage {
        type Error = super::error::ConversionError;
        fn try_from(
            value: AgentHelloMessage,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                device_id: value.device_id?,
                payload: value.payload?,
                protocol_version: value.protocol_version?,
                request_id: value.request_id?,
                session_id: value.session_id?,
                type_: value.type_?,
            })
        }
    }
    impl ::std::convert::From<super::AgentHelloMessage> for AgentHelloMessage {
        fn from(value: super::AgentHelloMessage) -> Self {
            Self {
                device_id: Ok(value.device_id),
                payload: Ok(value.payload),
                protocol_version: Ok(value.protocol_version),
                request_id: Ok(value.request_id),
                session_id: Ok(value.session_id),
                type_: Ok(value.type_),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct AgentHelloMessagePayload {
        agent_version: ::std::result::Result<super::Semver, ::std::string::String>,
        capabilities: ::std::result::Result<Vec<super::Capability>, ::std::string::String>,
        platform: ::std::result::Result<super::Platform, ::std::string::String>,
    }
    impl ::std::default::Default for AgentHelloMessagePayload {
        fn default() -> Self {
            Self {
                agent_version: Err("no value supplied for agent_version".to_string()),
                capabilities: Err("no value supplied for capabilities".to_string()),
                platform: Err("no value supplied for platform".to_string()),
            }
        }
    }
    impl AgentHelloMessagePayload {
        pub fn agent_version<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Semver>,
            T::Error: ::std::fmt::Display,
        {
            self.agent_version = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for agent_version: {e}"));
            self
        }
        pub fn capabilities<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<Vec<super::Capability>>,
            T::Error: ::std::fmt::Display,
        {
            self.capabilities = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for capabilities: {e}"));
            self
        }
        pub fn platform<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Platform>,
            T::Error: ::std::fmt::Display,
        {
            self.platform = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for platform: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<AgentHelloMessagePayload> for super::AgentHelloMessagePayload {
        type Error = super::error::ConversionError;
        fn try_from(
            value: AgentHelloMessagePayload,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                agent_version: value.agent_version?,
                capabilities: value.capabilities?,
                platform: value.platform?,
            })
        }
    }
    impl ::std::convert::From<super::AgentHelloMessagePayload> for AgentHelloMessagePayload {
        fn from(value: super::AgentHelloMessagePayload) -> Self {
            Self {
                agent_version: Ok(value.agent_version),
                capabilities: Ok(value.capabilities),
                platform: Ok(value.platform),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct FileEntry {
        index: ::std::result::Result<super::FileIndex, ::std::string::String>,
        relative_path: ::std::result::Result<super::SafeRelativePath, ::std::string::String>,
        size: ::std::result::Result<super::FileSize, ::std::string::String>,
    }
    impl ::std::default::Default for FileEntry {
        fn default() -> Self {
            Self {
                index: Err("no value supplied for index".to_string()),
                relative_path: Err("no value supplied for relative_path".to_string()),
                size: Err("no value supplied for size".to_string()),
            }
        }
    }
    impl FileEntry {
        pub fn index<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::FileIndex>,
            T::Error: ::std::fmt::Display,
        {
            self.index = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for index: {e}"));
            self
        }
        pub fn relative_path<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::SafeRelativePath>,
            T::Error: ::std::fmt::Display,
        {
            self.relative_path = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for relative_path: {e}"));
            self
        }
        pub fn size<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::FileSize>,
            T::Error: ::std::fmt::Display,
        {
            self.size = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for size: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<FileEntry> for super::FileEntry {
        type Error = super::error::ConversionError;
        fn try_from(
            value: FileEntry,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                index: value.index?,
                relative_path: value.relative_path?,
                size: value.size?,
            })
        }
    }
    impl ::std::convert::From<super::FileEntry> for FileEntry {
        fn from(value: super::FileEntry) -> Self {
            Self {
                index: Ok(value.index),
                relative_path: Ok(value.relative_path),
                size: Ok(value.size),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TerminalCloseMessage {
        device_id: ::std::result::Result<super::Uuid, ::std::string::String>,
        payload: ::std::result::Result<super::TerminalCloseMessagePayload, ::std::string::String>,
        protocol_version: ::std::result::Result<
            super::TerminalCloseMessageProtocolVersion,
            ::std::string::String,
        >,
        request_id: ::std::result::Result<(), ::std::string::String>,
        session_id: ::std::result::Result<super::Uuid, ::std::string::String>,
        type_: ::std::result::Result<super::TerminalCloseMessageType, ::std::string::String>,
    }
    impl ::std::default::Default for TerminalCloseMessage {
        fn default() -> Self {
            Self {
                device_id: Err("no value supplied for device_id".to_string()),
                payload: Err("no value supplied for payload".to_string()),
                protocol_version: Err("no value supplied for protocol_version".to_string()),
                request_id: Err("no value supplied for request_id".to_string()),
                session_id: Err("no value supplied for session_id".to_string()),
                type_: Err("no value supplied for type_".to_string()),
            }
        }
    }
    impl TerminalCloseMessage {
        pub fn device_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.device_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for device_id: {e}"));
            self
        }
        pub fn payload<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TerminalCloseMessagePayload>,
            T::Error: ::std::fmt::Display,
        {
            self.payload = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for payload: {e}"));
            self
        }
        pub fn protocol_version<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TerminalCloseMessageProtocolVersion>,
            T::Error: ::std::fmt::Display,
        {
            self.protocol_version = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for protocol_version: {e}"));
            self
        }
        pub fn request_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<()>,
            T::Error: ::std::fmt::Display,
        {
            self.request_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for request_id: {e}"));
            self
        }
        pub fn session_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.session_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for session_id: {e}"));
            self
        }
        pub fn type_<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TerminalCloseMessageType>,
            T::Error: ::std::fmt::Display,
        {
            self.type_ = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for type_: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<TerminalCloseMessage> for super::TerminalCloseMessage {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TerminalCloseMessage,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                device_id: value.device_id?,
                payload: value.payload?,
                protocol_version: value.protocol_version?,
                request_id: value.request_id?,
                session_id: value.session_id?,
                type_: value.type_?,
            })
        }
    }
    impl ::std::convert::From<super::TerminalCloseMessage> for TerminalCloseMessage {
        fn from(value: super::TerminalCloseMessage) -> Self {
            Self {
                device_id: Ok(value.device_id),
                payload: Ok(value.payload),
                protocol_version: Ok(value.protocol_version),
                request_id: Ok(value.request_id),
                session_id: Ok(value.session_id),
                type_: Ok(value.type_),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TerminalCloseMessagePayload {
        exit_code: ::std::result::Result<::std::option::Option<i32>, ::std::string::String>,
        reason: ::std::result::Result<super::TerminalCloseReason, ::std::string::String>,
    }
    impl ::std::default::Default for TerminalCloseMessagePayload {
        fn default() -> Self {
            Self {
                exit_code: Err("no value supplied for exit_code".to_string()),
                reason: Err("no value supplied for reason".to_string()),
            }
        }
    }
    impl TerminalCloseMessagePayload {
        pub fn exit_code<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<i32>>,
            T::Error: ::std::fmt::Display,
        {
            self.exit_code = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for exit_code: {e}"));
            self
        }
        pub fn reason<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TerminalCloseReason>,
            T::Error: ::std::fmt::Display,
        {
            self.reason = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for reason: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<TerminalCloseMessagePayload> for super::TerminalCloseMessagePayload {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TerminalCloseMessagePayload,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                exit_code: value.exit_code?,
                reason: value.reason?,
            })
        }
    }
    impl ::std::convert::From<super::TerminalCloseMessagePayload> for TerminalCloseMessagePayload {
        fn from(value: super::TerminalCloseMessagePayload) -> Self {
            Self {
                exit_code: Ok(value.exit_code),
                reason: Ok(value.reason),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TerminalErrorMessage {
        device_id: ::std::result::Result<super::Uuid, ::std::string::String>,
        payload: ::std::result::Result<super::TerminalErrorMessagePayload, ::std::string::String>,
        protocol_version: ::std::result::Result<
            super::TerminalErrorMessageProtocolVersion,
            ::std::string::String,
        >,
        request_id: ::std::result::Result<super::UuidOrNull, ::std::string::String>,
        session_id: ::std::result::Result<super::UuidOrNull, ::std::string::String>,
        type_: ::std::result::Result<super::TerminalErrorMessageType, ::std::string::String>,
    }
    impl ::std::default::Default for TerminalErrorMessage {
        fn default() -> Self {
            Self {
                device_id: Err("no value supplied for device_id".to_string()),
                payload: Err("no value supplied for payload".to_string()),
                protocol_version: Err("no value supplied for protocol_version".to_string()),
                request_id: Err("no value supplied for request_id".to_string()),
                session_id: Err("no value supplied for session_id".to_string()),
                type_: Err("no value supplied for type_".to_string()),
            }
        }
    }
    impl TerminalErrorMessage {
        pub fn device_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.device_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for device_id: {e}"));
            self
        }
        pub fn payload<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TerminalErrorMessagePayload>,
            T::Error: ::std::fmt::Display,
        {
            self.payload = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for payload: {e}"));
            self
        }
        pub fn protocol_version<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TerminalErrorMessageProtocolVersion>,
            T::Error: ::std::fmt::Display,
        {
            self.protocol_version = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for protocol_version: {e}"));
            self
        }
        pub fn request_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::UuidOrNull>,
            T::Error: ::std::fmt::Display,
        {
            self.request_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for request_id: {e}"));
            self
        }
        pub fn session_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::UuidOrNull>,
            T::Error: ::std::fmt::Display,
        {
            self.session_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for session_id: {e}"));
            self
        }
        pub fn type_<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TerminalErrorMessageType>,
            T::Error: ::std::fmt::Display,
        {
            self.type_ = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for type_: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<TerminalErrorMessage> for super::TerminalErrorMessage {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TerminalErrorMessage,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                device_id: value.device_id?,
                payload: value.payload?,
                protocol_version: value.protocol_version?,
                request_id: value.request_id?,
                session_id: value.session_id?,
                type_: value.type_?,
            })
        }
    }
    impl ::std::convert::From<super::TerminalErrorMessage> for TerminalErrorMessage {
        fn from(value: super::TerminalErrorMessage) -> Self {
            Self {
                device_id: Ok(value.device_id),
                payload: Ok(value.payload),
                protocol_version: Ok(value.protocol_version),
                request_id: Ok(value.request_id),
                session_id: Ok(value.session_id),
                type_: Ok(value.type_),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TerminalErrorMessagePayload {
        code: ::std::result::Result<super::ErrorCode, ::std::string::String>,
        message:
            ::std::result::Result<super::TerminalErrorMessagePayloadMessage, ::std::string::String>,
    }
    impl ::std::default::Default for TerminalErrorMessagePayload {
        fn default() -> Self {
            Self {
                code: Err("no value supplied for code".to_string()),
                message: Err("no value supplied for message".to_string()),
            }
        }
    }
    impl TerminalErrorMessagePayload {
        pub fn code<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ErrorCode>,
            T::Error: ::std::fmt::Display,
        {
            self.code = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for code: {e}"));
            self
        }
        pub fn message<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TerminalErrorMessagePayloadMessage>,
            T::Error: ::std::fmt::Display,
        {
            self.message = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for message: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<TerminalErrorMessagePayload> for super::TerminalErrorMessagePayload {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TerminalErrorMessagePayload,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                code: value.code?,
                message: value.message?,
            })
        }
    }
    impl ::std::convert::From<super::TerminalErrorMessagePayload> for TerminalErrorMessagePayload {
        fn from(value: super::TerminalErrorMessagePayload) -> Self {
            Self {
                code: Ok(value.code),
                message: Ok(value.message),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TerminalOpenMessage {
        device_id: ::std::result::Result<super::Uuid, ::std::string::String>,
        payload: ::std::result::Result<super::TerminalOpenMessagePayload, ::std::string::String>,
        protocol_version:
            ::std::result::Result<super::TerminalOpenMessageProtocolVersion, ::std::string::String>,
        request_id: ::std::result::Result<super::Uuid, ::std::string::String>,
        session_id: ::std::result::Result<(), ::std::string::String>,
        type_: ::std::result::Result<super::TerminalOpenMessageType, ::std::string::String>,
    }
    impl ::std::default::Default for TerminalOpenMessage {
        fn default() -> Self {
            Self {
                device_id: Err("no value supplied for device_id".to_string()),
                payload: Err("no value supplied for payload".to_string()),
                protocol_version: Err("no value supplied for protocol_version".to_string()),
                request_id: Err("no value supplied for request_id".to_string()),
                session_id: Err("no value supplied for session_id".to_string()),
                type_: Err("no value supplied for type_".to_string()),
            }
        }
    }
    impl TerminalOpenMessage {
        pub fn device_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.device_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for device_id: {e}"));
            self
        }
        pub fn payload<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TerminalOpenMessagePayload>,
            T::Error: ::std::fmt::Display,
        {
            self.payload = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for payload: {e}"));
            self
        }
        pub fn protocol_version<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TerminalOpenMessageProtocolVersion>,
            T::Error: ::std::fmt::Display,
        {
            self.protocol_version = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for protocol_version: {e}"));
            self
        }
        pub fn request_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.request_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for request_id: {e}"));
            self
        }
        pub fn session_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<()>,
            T::Error: ::std::fmt::Display,
        {
            self.session_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for session_id: {e}"));
            self
        }
        pub fn type_<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TerminalOpenMessageType>,
            T::Error: ::std::fmt::Display,
        {
            self.type_ = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for type_: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<TerminalOpenMessage> for super::TerminalOpenMessage {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TerminalOpenMessage,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                device_id: value.device_id?,
                payload: value.payload?,
                protocol_version: value.protocol_version?,
                request_id: value.request_id?,
                session_id: value.session_id?,
                type_: value.type_?,
            })
        }
    }
    impl ::std::convert::From<super::TerminalOpenMessage> for TerminalOpenMessage {
        fn from(value: super::TerminalOpenMessage) -> Self {
            Self {
                device_id: Ok(value.device_id),
                payload: Ok(value.payload),
                protocol_version: Ok(value.protocol_version),
                request_id: Ok(value.request_id),
                session_id: Ok(value.session_id),
                type_: Ok(value.type_),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TerminalOpenMessagePayload {
        cols: ::std::result::Result<super::Cols, ::std::string::String>,
        rows: ::std::result::Result<super::Rows, ::std::string::String>,
    }
    impl ::std::default::Default for TerminalOpenMessagePayload {
        fn default() -> Self {
            Self {
                cols: Err("no value supplied for cols".to_string()),
                rows: Err("no value supplied for rows".to_string()),
            }
        }
    }
    impl TerminalOpenMessagePayload {
        pub fn cols<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Cols>,
            T::Error: ::std::fmt::Display,
        {
            self.cols = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for cols: {e}"));
            self
        }
        pub fn rows<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Rows>,
            T::Error: ::std::fmt::Display,
        {
            self.rows = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for rows: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<TerminalOpenMessagePayload> for super::TerminalOpenMessagePayload {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TerminalOpenMessagePayload,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                cols: value.cols?,
                rows: value.rows?,
            })
        }
    }
    impl ::std::convert::From<super::TerminalOpenMessagePayload> for TerminalOpenMessagePayload {
        fn from(value: super::TerminalOpenMessagePayload) -> Self {
            Self {
                cols: Ok(value.cols),
                rows: Ok(value.rows),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TerminalOpenedMessage {
        device_id: ::std::result::Result<super::Uuid, ::std::string::String>,
        payload: ::std::result::Result<super::TerminalOpenedMessagePayload, ::std::string::String>,
        protocol_version: ::std::result::Result<
            super::TerminalOpenedMessageProtocolVersion,
            ::std::string::String,
        >,
        request_id: ::std::result::Result<super::Uuid, ::std::string::String>,
        session_id: ::std::result::Result<super::Uuid, ::std::string::String>,
        type_: ::std::result::Result<super::TerminalOpenedMessageType, ::std::string::String>,
    }
    impl ::std::default::Default for TerminalOpenedMessage {
        fn default() -> Self {
            Self {
                device_id: Err("no value supplied for device_id".to_string()),
                payload: Err("no value supplied for payload".to_string()),
                protocol_version: Err("no value supplied for protocol_version".to_string()),
                request_id: Err("no value supplied for request_id".to_string()),
                session_id: Err("no value supplied for session_id".to_string()),
                type_: Err("no value supplied for type_".to_string()),
            }
        }
    }
    impl TerminalOpenedMessage {
        pub fn device_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.device_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for device_id: {e}"));
            self
        }
        pub fn payload<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TerminalOpenedMessagePayload>,
            T::Error: ::std::fmt::Display,
        {
            self.payload = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for payload: {e}"));
            self
        }
        pub fn protocol_version<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TerminalOpenedMessageProtocolVersion>,
            T::Error: ::std::fmt::Display,
        {
            self.protocol_version = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for protocol_version: {e}"));
            self
        }
        pub fn request_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.request_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for request_id: {e}"));
            self
        }
        pub fn session_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.session_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for session_id: {e}"));
            self
        }
        pub fn type_<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TerminalOpenedMessageType>,
            T::Error: ::std::fmt::Display,
        {
            self.type_ = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for type_: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<TerminalOpenedMessage> for super::TerminalOpenedMessage {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TerminalOpenedMessage,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                device_id: value.device_id?,
                payload: value.payload?,
                protocol_version: value.protocol_version?,
                request_id: value.request_id?,
                session_id: value.session_id?,
                type_: value.type_?,
            })
        }
    }
    impl ::std::convert::From<super::TerminalOpenedMessage> for TerminalOpenedMessage {
        fn from(value: super::TerminalOpenedMessage) -> Self {
            Self {
                device_id: Ok(value.device_id),
                payload: Ok(value.payload),
                protocol_version: Ok(value.protocol_version),
                request_id: Ok(value.request_id),
                session_id: Ok(value.session_id),
                type_: Ok(value.type_),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TerminalOpenedMessagePayload {
        shell:
            ::std::result::Result<super::TerminalOpenedMessagePayloadShell, ::std::string::String>,
    }
    impl ::std::default::Default for TerminalOpenedMessagePayload {
        fn default() -> Self {
            Self {
                shell: Err("no value supplied for shell".to_string()),
            }
        }
    }
    impl TerminalOpenedMessagePayload {
        pub fn shell<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TerminalOpenedMessagePayloadShell>,
            T::Error: ::std::fmt::Display,
        {
            self.shell = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for shell: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<TerminalOpenedMessagePayload> for super::TerminalOpenedMessagePayload {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TerminalOpenedMessagePayload,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                shell: value.shell?,
            })
        }
    }
    impl ::std::convert::From<super::TerminalOpenedMessagePayload> for TerminalOpenedMessagePayload {
        fn from(value: super::TerminalOpenedMessagePayload) -> Self {
            Self {
                shell: Ok(value.shell),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TerminalResizeMessage {
        device_id: ::std::result::Result<super::Uuid, ::std::string::String>,
        payload: ::std::result::Result<super::TerminalResizeMessagePayload, ::std::string::String>,
        protocol_version: ::std::result::Result<
            super::TerminalResizeMessageProtocolVersion,
            ::std::string::String,
        >,
        request_id: ::std::result::Result<(), ::std::string::String>,
        session_id: ::std::result::Result<super::Uuid, ::std::string::String>,
        type_: ::std::result::Result<super::TerminalResizeMessageType, ::std::string::String>,
    }
    impl ::std::default::Default for TerminalResizeMessage {
        fn default() -> Self {
            Self {
                device_id: Err("no value supplied for device_id".to_string()),
                payload: Err("no value supplied for payload".to_string()),
                protocol_version: Err("no value supplied for protocol_version".to_string()),
                request_id: Err("no value supplied for request_id".to_string()),
                session_id: Err("no value supplied for session_id".to_string()),
                type_: Err("no value supplied for type_".to_string()),
            }
        }
    }
    impl TerminalResizeMessage {
        pub fn device_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.device_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for device_id: {e}"));
            self
        }
        pub fn payload<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TerminalResizeMessagePayload>,
            T::Error: ::std::fmt::Display,
        {
            self.payload = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for payload: {e}"));
            self
        }
        pub fn protocol_version<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TerminalResizeMessageProtocolVersion>,
            T::Error: ::std::fmt::Display,
        {
            self.protocol_version = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for protocol_version: {e}"));
            self
        }
        pub fn request_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<()>,
            T::Error: ::std::fmt::Display,
        {
            self.request_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for request_id: {e}"));
            self
        }
        pub fn session_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.session_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for session_id: {e}"));
            self
        }
        pub fn type_<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TerminalResizeMessageType>,
            T::Error: ::std::fmt::Display,
        {
            self.type_ = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for type_: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<TerminalResizeMessage> for super::TerminalResizeMessage {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TerminalResizeMessage,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                device_id: value.device_id?,
                payload: value.payload?,
                protocol_version: value.protocol_version?,
                request_id: value.request_id?,
                session_id: value.session_id?,
                type_: value.type_?,
            })
        }
    }
    impl ::std::convert::From<super::TerminalResizeMessage> for TerminalResizeMessage {
        fn from(value: super::TerminalResizeMessage) -> Self {
            Self {
                device_id: Ok(value.device_id),
                payload: Ok(value.payload),
                protocol_version: Ok(value.protocol_version),
                request_id: Ok(value.request_id),
                session_id: Ok(value.session_id),
                type_: Ok(value.type_),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TerminalResizeMessagePayload {
        cols: ::std::result::Result<super::Cols, ::std::string::String>,
        rows: ::std::result::Result<super::Rows, ::std::string::String>,
    }
    impl ::std::default::Default for TerminalResizeMessagePayload {
        fn default() -> Self {
            Self {
                cols: Err("no value supplied for cols".to_string()),
                rows: Err("no value supplied for rows".to_string()),
            }
        }
    }
    impl TerminalResizeMessagePayload {
        pub fn cols<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Cols>,
            T::Error: ::std::fmt::Display,
        {
            self.cols = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for cols: {e}"));
            self
        }
        pub fn rows<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Rows>,
            T::Error: ::std::fmt::Display,
        {
            self.rows = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for rows: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<TerminalResizeMessagePayload> for super::TerminalResizeMessagePayload {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TerminalResizeMessagePayload,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                cols: value.cols?,
                rows: value.rows?,
            })
        }
    }
    impl ::std::convert::From<super::TerminalResizeMessagePayload> for TerminalResizeMessagePayload {
        fn from(value: super::TerminalResizeMessagePayload) -> Self {
            Self {
                cols: Ok(value.cols),
                rows: Ok(value.rows),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TerminalShellEventMessage {
        device_id: ::std::result::Result<super::Uuid, ::std::string::String>,
        payload:
            ::std::result::Result<super::TerminalShellEventMessagePayload, ::std::string::String>,
        protocol_version: ::std::result::Result<
            super::TerminalShellEventMessageProtocolVersion,
            ::std::string::String,
        >,
        request_id: ::std::result::Result<(), ::std::string::String>,
        session_id: ::std::result::Result<super::Uuid, ::std::string::String>,
        type_: ::std::result::Result<super::TerminalShellEventMessageType, ::std::string::String>,
    }
    impl ::std::default::Default for TerminalShellEventMessage {
        fn default() -> Self {
            Self {
                device_id: Err("no value supplied for device_id".to_string()),
                payload: Err("no value supplied for payload".to_string()),
                protocol_version: Err("no value supplied for protocol_version".to_string()),
                request_id: Err("no value supplied for request_id".to_string()),
                session_id: Err("no value supplied for session_id".to_string()),
                type_: Err("no value supplied for type_".to_string()),
            }
        }
    }
    impl TerminalShellEventMessage {
        pub fn device_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.device_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for device_id: {e}"));
            self
        }
        pub fn payload<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TerminalShellEventMessagePayload>,
            T::Error: ::std::fmt::Display,
        {
            self.payload = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for payload: {e}"));
            self
        }
        pub fn protocol_version<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TerminalShellEventMessageProtocolVersion>,
            T::Error: ::std::fmt::Display,
        {
            self.protocol_version = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for protocol_version: {e}"));
            self
        }
        pub fn request_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<()>,
            T::Error: ::std::fmt::Display,
        {
            self.request_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for request_id: {e}"));
            self
        }
        pub fn session_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.session_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for session_id: {e}"));
            self
        }
        pub fn type_<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TerminalShellEventMessageType>,
            T::Error: ::std::fmt::Display,
        {
            self.type_ = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for type_: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<TerminalShellEventMessage> for super::TerminalShellEventMessage {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TerminalShellEventMessage,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                device_id: value.device_id?,
                payload: value.payload?,
                protocol_version: value.protocol_version?,
                request_id: value.request_id?,
                session_id: value.session_id?,
                type_: value.type_?,
            })
        }
    }
    impl ::std::convert::From<super::TerminalShellEventMessage> for TerminalShellEventMessage {
        fn from(value: super::TerminalShellEventMessage) -> Self {
            Self {
                device_id: Ok(value.device_id),
                payload: Ok(value.payload),
                protocol_version: Ok(value.protocol_version),
                request_id: Ok(value.request_id),
                session_id: Ok(value.session_id),
                type_: Ok(value.type_),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TerminalShellEventMessagePayload {
        exit_code: ::std::result::Result<::std::option::Option<i32>, ::std::string::String>,
        source: ::std::result::Result<super::ShellEventSource, ::std::string::String>,
        type_: ::std::result::Result<super::ShellEventName, ::std::string::String>,
    }
    impl ::std::default::Default for TerminalShellEventMessagePayload {
        fn default() -> Self {
            Self {
                exit_code: Err("no value supplied for exit_code".to_string()),
                source: Err("no value supplied for source".to_string()),
                type_: Err("no value supplied for type_".to_string()),
            }
        }
    }
    impl TerminalShellEventMessagePayload {
        pub fn exit_code<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<i32>>,
            T::Error: ::std::fmt::Display,
        {
            self.exit_code = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for exit_code: {e}"));
            self
        }
        pub fn source<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ShellEventSource>,
            T::Error: ::std::fmt::Display,
        {
            self.source = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for source: {e}"));
            self
        }
        pub fn type_<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ShellEventName>,
            T::Error: ::std::fmt::Display,
        {
            self.type_ = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for type_: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<TerminalShellEventMessagePayload>
        for super::TerminalShellEventMessagePayload
    {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TerminalShellEventMessagePayload,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                exit_code: value.exit_code?,
                source: value.source?,
                type_: value.type_?,
            })
        }
    }
    impl ::std::convert::From<super::TerminalShellEventMessagePayload>
        for TerminalShellEventMessagePayload
    {
        fn from(value: super::TerminalShellEventMessagePayload) -> Self {
            Self {
                exit_code: Ok(value.exit_code),
                source: Ok(value.source),
                type_: Ok(value.type_),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TransferAbortMessage {
        device_id: ::std::result::Result<super::Uuid, ::std::string::String>,
        payload: ::std::result::Result<super::TransferAbortMessagePayload, ::std::string::String>,
        protocol_version: ::std::result::Result<
            super::TransferAbortMessageProtocolVersion,
            ::std::string::String,
        >,
        request_id: ::std::result::Result<(), ::std::string::String>,
        session_id: ::std::result::Result<(), ::std::string::String>,
        type_: ::std::result::Result<super::TransferAbortMessageType, ::std::string::String>,
    }
    impl ::std::default::Default for TransferAbortMessage {
        fn default() -> Self {
            Self {
                device_id: Err("no value supplied for device_id".to_string()),
                payload: Err("no value supplied for payload".to_string()),
                protocol_version: Err("no value supplied for protocol_version".to_string()),
                request_id: Err("no value supplied for request_id".to_string()),
                session_id: Err("no value supplied for session_id".to_string()),
                type_: Err("no value supplied for type_".to_string()),
            }
        }
    }
    impl TransferAbortMessage {
        pub fn device_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.device_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for device_id: {e}"));
            self
        }
        pub fn payload<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TransferAbortMessagePayload>,
            T::Error: ::std::fmt::Display,
        {
            self.payload = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for payload: {e}"));
            self
        }
        pub fn protocol_version<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TransferAbortMessageProtocolVersion>,
            T::Error: ::std::fmt::Display,
        {
            self.protocol_version = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for protocol_version: {e}"));
            self
        }
        pub fn request_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<()>,
            T::Error: ::std::fmt::Display,
        {
            self.request_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for request_id: {e}"));
            self
        }
        pub fn session_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<()>,
            T::Error: ::std::fmt::Display,
        {
            self.session_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for session_id: {e}"));
            self
        }
        pub fn type_<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TransferAbortMessageType>,
            T::Error: ::std::fmt::Display,
        {
            self.type_ = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for type_: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<TransferAbortMessage> for super::TransferAbortMessage {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TransferAbortMessage,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                device_id: value.device_id?,
                payload: value.payload?,
                protocol_version: value.protocol_version?,
                request_id: value.request_id?,
                session_id: value.session_id?,
                type_: value.type_?,
            })
        }
    }
    impl ::std::convert::From<super::TransferAbortMessage> for TransferAbortMessage {
        fn from(value: super::TransferAbortMessage) -> Self {
            Self {
                device_id: Ok(value.device_id),
                payload: Ok(value.payload),
                protocol_version: Ok(value.protocol_version),
                request_id: Ok(value.request_id),
                session_id: Ok(value.session_id),
                type_: Ok(value.type_),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TransferAbortMessagePayload {
        code: ::std::result::Result<super::ErrorCode, ::std::string::String>,
        transfer_id: ::std::result::Result<super::Uuid, ::std::string::String>,
    }
    impl ::std::default::Default for TransferAbortMessagePayload {
        fn default() -> Self {
            Self {
                code: Err("no value supplied for code".to_string()),
                transfer_id: Err("no value supplied for transfer_id".to_string()),
            }
        }
    }
    impl TransferAbortMessagePayload {
        pub fn code<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::ErrorCode>,
            T::Error: ::std::fmt::Display,
        {
            self.code = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for code: {e}"));
            self
        }
        pub fn transfer_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.transfer_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for transfer_id: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<TransferAbortMessagePayload> for super::TransferAbortMessagePayload {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TransferAbortMessagePayload,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                code: value.code?,
                transfer_id: value.transfer_id?,
            })
        }
    }
    impl ::std::convert::From<super::TransferAbortMessagePayload> for TransferAbortMessagePayload {
        fn from(value: super::TransferAbortMessagePayload) -> Self {
            Self {
                code: Ok(value.code),
                transfer_id: Ok(value.transfer_id),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TransferAcceptedMessage {
        device_id: ::std::result::Result<super::Uuid, ::std::string::String>,
        payload:
            ::std::result::Result<super::TransferAcceptedMessagePayload, ::std::string::String>,
        protocol_version: ::std::result::Result<
            super::TransferAcceptedMessageProtocolVersion,
            ::std::string::String,
        >,
        request_id: ::std::result::Result<super::Uuid, ::std::string::String>,
        session_id: ::std::result::Result<(), ::std::string::String>,
        type_: ::std::result::Result<super::TransferAcceptedMessageType, ::std::string::String>,
    }
    impl ::std::default::Default for TransferAcceptedMessage {
        fn default() -> Self {
            Self {
                device_id: Err("no value supplied for device_id".to_string()),
                payload: Err("no value supplied for payload".to_string()),
                protocol_version: Err("no value supplied for protocol_version".to_string()),
                request_id: Err("no value supplied for request_id".to_string()),
                session_id: Err("no value supplied for session_id".to_string()),
                type_: Err("no value supplied for type_".to_string()),
            }
        }
    }
    impl TransferAcceptedMessage {
        pub fn device_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.device_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for device_id: {e}"));
            self
        }
        pub fn payload<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TransferAcceptedMessagePayload>,
            T::Error: ::std::fmt::Display,
        {
            self.payload = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for payload: {e}"));
            self
        }
        pub fn protocol_version<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TransferAcceptedMessageProtocolVersion>,
            T::Error: ::std::fmt::Display,
        {
            self.protocol_version = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for protocol_version: {e}"));
            self
        }
        pub fn request_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.request_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for request_id: {e}"));
            self
        }
        pub fn session_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<()>,
            T::Error: ::std::fmt::Display,
        {
            self.session_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for session_id: {e}"));
            self
        }
        pub fn type_<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TransferAcceptedMessageType>,
            T::Error: ::std::fmt::Display,
        {
            self.type_ = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for type_: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<TransferAcceptedMessage> for super::TransferAcceptedMessage {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TransferAcceptedMessage,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                device_id: value.device_id?,
                payload: value.payload?,
                protocol_version: value.protocol_version?,
                request_id: value.request_id?,
                session_id: value.session_id?,
                type_: value.type_?,
            })
        }
    }
    impl ::std::convert::From<super::TransferAcceptedMessage> for TransferAcceptedMessage {
        fn from(value: super::TransferAcceptedMessage) -> Self {
            Self {
                device_id: Ok(value.device_id),
                payload: Ok(value.payload),
                protocol_version: Ok(value.protocol_version),
                request_id: Ok(value.request_id),
                session_id: Ok(value.session_id),
                type_: Ok(value.type_),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TransferAcceptedMessagePayload {
        granted_bytes: ::std::result::Result<i64, ::std::string::String>,
        transfer_id: ::std::result::Result<super::Uuid, ::std::string::String>,
    }
    impl ::std::default::Default for TransferAcceptedMessagePayload {
        fn default() -> Self {
            Self {
                granted_bytes: Err("no value supplied for granted_bytes".to_string()),
                transfer_id: Err("no value supplied for transfer_id".to_string()),
            }
        }
    }
    impl TransferAcceptedMessagePayload {
        pub fn granted_bytes<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.granted_bytes = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for granted_bytes: {e}"));
            self
        }
        pub fn transfer_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.transfer_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for transfer_id: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<TransferAcceptedMessagePayload>
        for super::TransferAcceptedMessagePayload
    {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TransferAcceptedMessagePayload,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                granted_bytes: value.granted_bytes?,
                transfer_id: value.transfer_id?,
            })
        }
    }
    impl ::std::convert::From<super::TransferAcceptedMessagePayload>
        for TransferAcceptedMessagePayload
    {
        fn from(value: super::TransferAcceptedMessagePayload) -> Self {
            Self {
                granted_bytes: Ok(value.granted_bytes),
                transfer_id: Ok(value.transfer_id),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TransferCompleteMessage {
        device_id: ::std::result::Result<super::Uuid, ::std::string::String>,
        payload:
            ::std::result::Result<super::TransferCompleteMessagePayload, ::std::string::String>,
        protocol_version: ::std::result::Result<
            super::TransferCompleteMessageProtocolVersion,
            ::std::string::String,
        >,
        request_id: ::std::result::Result<(), ::std::string::String>,
        session_id: ::std::result::Result<(), ::std::string::String>,
        type_: ::std::result::Result<super::TransferCompleteMessageType, ::std::string::String>,
    }
    impl ::std::default::Default for TransferCompleteMessage {
        fn default() -> Self {
            Self {
                device_id: Err("no value supplied for device_id".to_string()),
                payload: Err("no value supplied for payload".to_string()),
                protocol_version: Err("no value supplied for protocol_version".to_string()),
                request_id: Err("no value supplied for request_id".to_string()),
                session_id: Err("no value supplied for session_id".to_string()),
                type_: Err("no value supplied for type_".to_string()),
            }
        }
    }
    impl TransferCompleteMessage {
        pub fn device_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.device_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for device_id: {e}"));
            self
        }
        pub fn payload<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TransferCompleteMessagePayload>,
            T::Error: ::std::fmt::Display,
        {
            self.payload = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for payload: {e}"));
            self
        }
        pub fn protocol_version<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TransferCompleteMessageProtocolVersion>,
            T::Error: ::std::fmt::Display,
        {
            self.protocol_version = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for protocol_version: {e}"));
            self
        }
        pub fn request_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<()>,
            T::Error: ::std::fmt::Display,
        {
            self.request_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for request_id: {e}"));
            self
        }
        pub fn session_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<()>,
            T::Error: ::std::fmt::Display,
        {
            self.session_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for session_id: {e}"));
            self
        }
        pub fn type_<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TransferCompleteMessageType>,
            T::Error: ::std::fmt::Display,
        {
            self.type_ = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for type_: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<TransferCompleteMessage> for super::TransferCompleteMessage {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TransferCompleteMessage,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                device_id: value.device_id?,
                payload: value.payload?,
                protocol_version: value.protocol_version?,
                request_id: value.request_id?,
                session_id: value.session_id?,
                type_: value.type_?,
            })
        }
    }
    impl ::std::convert::From<super::TransferCompleteMessage> for TransferCompleteMessage {
        fn from(value: super::TransferCompleteMessage) -> Self {
            Self {
                device_id: Ok(value.device_id),
                payload: Ok(value.payload),
                protocol_version: Ok(value.protocol_version),
                request_id: Ok(value.request_id),
                session_id: Ok(value.session_id),
                type_: Ok(value.type_),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TransferCompleteMessagePayload {
        transfer_id: ::std::result::Result<super::Uuid, ::std::string::String>,
    }
    impl ::std::default::Default for TransferCompleteMessagePayload {
        fn default() -> Self {
            Self {
                transfer_id: Err("no value supplied for transfer_id".to_string()),
            }
        }
    }
    impl TransferCompleteMessagePayload {
        pub fn transfer_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.transfer_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for transfer_id: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<TransferCompleteMessagePayload>
        for super::TransferCompleteMessagePayload
    {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TransferCompleteMessagePayload,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                transfer_id: value.transfer_id?,
            })
        }
    }
    impl ::std::convert::From<super::TransferCompleteMessagePayload>
        for TransferCompleteMessagePayload
    {
        fn from(value: super::TransferCompleteMessagePayload) -> Self {
            Self {
                transfer_id: Ok(value.transfer_id),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TransferCreditMessage {
        device_id: ::std::result::Result<super::Uuid, ::std::string::String>,
        payload: ::std::result::Result<super::TransferCreditMessagePayload, ::std::string::String>,
        protocol_version: ::std::result::Result<
            super::TransferCreditMessageProtocolVersion,
            ::std::string::String,
        >,
        request_id: ::std::result::Result<(), ::std::string::String>,
        session_id: ::std::result::Result<(), ::std::string::String>,
        type_: ::std::result::Result<super::TransferCreditMessageType, ::std::string::String>,
    }
    impl ::std::default::Default for TransferCreditMessage {
        fn default() -> Self {
            Self {
                device_id: Err("no value supplied for device_id".to_string()),
                payload: Err("no value supplied for payload".to_string()),
                protocol_version: Err("no value supplied for protocol_version".to_string()),
                request_id: Err("no value supplied for request_id".to_string()),
                session_id: Err("no value supplied for session_id".to_string()),
                type_: Err("no value supplied for type_".to_string()),
            }
        }
    }
    impl TransferCreditMessage {
        pub fn device_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.device_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for device_id: {e}"));
            self
        }
        pub fn payload<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TransferCreditMessagePayload>,
            T::Error: ::std::fmt::Display,
        {
            self.payload = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for payload: {e}"));
            self
        }
        pub fn protocol_version<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TransferCreditMessageProtocolVersion>,
            T::Error: ::std::fmt::Display,
        {
            self.protocol_version = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for protocol_version: {e}"));
            self
        }
        pub fn request_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<()>,
            T::Error: ::std::fmt::Display,
        {
            self.request_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for request_id: {e}"));
            self
        }
        pub fn session_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<()>,
            T::Error: ::std::fmt::Display,
        {
            self.session_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for session_id: {e}"));
            self
        }
        pub fn type_<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TransferCreditMessageType>,
            T::Error: ::std::fmt::Display,
        {
            self.type_ = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for type_: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<TransferCreditMessage> for super::TransferCreditMessage {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TransferCreditMessage,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                device_id: value.device_id?,
                payload: value.payload?,
                protocol_version: value.protocol_version?,
                request_id: value.request_id?,
                session_id: value.session_id?,
                type_: value.type_?,
            })
        }
    }
    impl ::std::convert::From<super::TransferCreditMessage> for TransferCreditMessage {
        fn from(value: super::TransferCreditMessage) -> Self {
            Self {
                device_id: Ok(value.device_id),
                payload: Ok(value.payload),
                protocol_version: Ok(value.protocol_version),
                request_id: Ok(value.request_id),
                session_id: Ok(value.session_id),
                type_: Ok(value.type_),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TransferCreditMessagePayload {
        granted_bytes: ::std::result::Result<i64, ::std::string::String>,
        transfer_id: ::std::result::Result<super::Uuid, ::std::string::String>,
    }
    impl ::std::default::Default for TransferCreditMessagePayload {
        fn default() -> Self {
            Self {
                granted_bytes: Err("no value supplied for granted_bytes".to_string()),
                transfer_id: Err("no value supplied for transfer_id".to_string()),
            }
        }
    }
    impl TransferCreditMessagePayload {
        pub fn granted_bytes<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<i64>,
            T::Error: ::std::fmt::Display,
        {
            self.granted_bytes = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for granted_bytes: {e}"));
            self
        }
        pub fn transfer_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.transfer_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for transfer_id: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<TransferCreditMessagePayload> for super::TransferCreditMessagePayload {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TransferCreditMessagePayload,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                granted_bytes: value.granted_bytes?,
                transfer_id: value.transfer_id?,
            })
        }
    }
    impl ::std::convert::From<super::TransferCreditMessagePayload> for TransferCreditMessagePayload {
        fn from(value: super::TransferCreditMessagePayload) -> Self {
            Self {
                granted_bytes: Ok(value.granted_bytes),
                transfer_id: Ok(value.transfer_id),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TransferFileEndMessage {
        device_id: ::std::result::Result<super::Uuid, ::std::string::String>,
        payload: ::std::result::Result<super::TransferFileEndMessagePayload, ::std::string::String>,
        protocol_version: ::std::result::Result<
            super::TransferFileEndMessageProtocolVersion,
            ::std::string::String,
        >,
        request_id: ::std::result::Result<(), ::std::string::String>,
        session_id: ::std::result::Result<(), ::std::string::String>,
        type_: ::std::result::Result<super::TransferFileEndMessageType, ::std::string::String>,
    }
    impl ::std::default::Default for TransferFileEndMessage {
        fn default() -> Self {
            Self {
                device_id: Err("no value supplied for device_id".to_string()),
                payload: Err("no value supplied for payload".to_string()),
                protocol_version: Err("no value supplied for protocol_version".to_string()),
                request_id: Err("no value supplied for request_id".to_string()),
                session_id: Err("no value supplied for session_id".to_string()),
                type_: Err("no value supplied for type_".to_string()),
            }
        }
    }
    impl TransferFileEndMessage {
        pub fn device_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.device_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for device_id: {e}"));
            self
        }
        pub fn payload<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TransferFileEndMessagePayload>,
            T::Error: ::std::fmt::Display,
        {
            self.payload = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for payload: {e}"));
            self
        }
        pub fn protocol_version<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TransferFileEndMessageProtocolVersion>,
            T::Error: ::std::fmt::Display,
        {
            self.protocol_version = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for protocol_version: {e}"));
            self
        }
        pub fn request_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<()>,
            T::Error: ::std::fmt::Display,
        {
            self.request_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for request_id: {e}"));
            self
        }
        pub fn session_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<()>,
            T::Error: ::std::fmt::Display,
        {
            self.session_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for session_id: {e}"));
            self
        }
        pub fn type_<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TransferFileEndMessageType>,
            T::Error: ::std::fmt::Display,
        {
            self.type_ = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for type_: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<TransferFileEndMessage> for super::TransferFileEndMessage {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TransferFileEndMessage,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                device_id: value.device_id?,
                payload: value.payload?,
                protocol_version: value.protocol_version?,
                request_id: value.request_id?,
                session_id: value.session_id?,
                type_: value.type_?,
            })
        }
    }
    impl ::std::convert::From<super::TransferFileEndMessage> for TransferFileEndMessage {
        fn from(value: super::TransferFileEndMessage) -> Self {
            Self {
                device_id: Ok(value.device_id),
                payload: Ok(value.payload),
                protocol_version: Ok(value.protocol_version),
                request_id: Ok(value.request_id),
                session_id: Ok(value.session_id),
                type_: Ok(value.type_),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TransferFileEndMessagePayload {
        file_index: ::std::result::Result<super::FileIndex, ::std::string::String>,
        sent_size: ::std::result::Result<super::FileSize, ::std::string::String>,
        transfer_id: ::std::result::Result<super::Uuid, ::std::string::String>,
    }
    impl ::std::default::Default for TransferFileEndMessagePayload {
        fn default() -> Self {
            Self {
                file_index: Err("no value supplied for file_index".to_string()),
                sent_size: Err("no value supplied for sent_size".to_string()),
                transfer_id: Err("no value supplied for transfer_id".to_string()),
            }
        }
    }
    impl TransferFileEndMessagePayload {
        pub fn file_index<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::FileIndex>,
            T::Error: ::std::fmt::Display,
        {
            self.file_index = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for file_index: {e}"));
            self
        }
        pub fn sent_size<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::FileSize>,
            T::Error: ::std::fmt::Display,
        {
            self.sent_size = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for sent_size: {e}"));
            self
        }
        pub fn transfer_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.transfer_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for transfer_id: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<TransferFileEndMessagePayload>
        for super::TransferFileEndMessagePayload
    {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TransferFileEndMessagePayload,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                file_index: value.file_index?,
                sent_size: value.sent_size?,
                transfer_id: value.transfer_id?,
            })
        }
    }
    impl ::std::convert::From<super::TransferFileEndMessagePayload> for TransferFileEndMessagePayload {
        fn from(value: super::TransferFileEndMessagePayload) -> Self {
            Self {
                file_index: Ok(value.file_index),
                sent_size: Ok(value.sent_size),
                transfer_id: Ok(value.transfer_id),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TransferResultMessage {
        device_id: ::std::result::Result<super::Uuid, ::std::string::String>,
        payload: ::std::result::Result<super::TransferResultMessagePayload, ::std::string::String>,
        protocol_version: ::std::result::Result<
            super::TransferResultMessageProtocolVersion,
            ::std::string::String,
        >,
        request_id: ::std::result::Result<(), ::std::string::String>,
        session_id: ::std::result::Result<(), ::std::string::String>,
        type_: ::std::result::Result<super::TransferResultMessageType, ::std::string::String>,
    }
    impl ::std::default::Default for TransferResultMessage {
        fn default() -> Self {
            Self {
                device_id: Err("no value supplied for device_id".to_string()),
                payload: Err("no value supplied for payload".to_string()),
                protocol_version: Err("no value supplied for protocol_version".to_string()),
                request_id: Err("no value supplied for request_id".to_string()),
                session_id: Err("no value supplied for session_id".to_string()),
                type_: Err("no value supplied for type_".to_string()),
            }
        }
    }
    impl TransferResultMessage {
        pub fn device_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.device_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for device_id: {e}"));
            self
        }
        pub fn payload<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TransferResultMessagePayload>,
            T::Error: ::std::fmt::Display,
        {
            self.payload = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for payload: {e}"));
            self
        }
        pub fn protocol_version<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TransferResultMessageProtocolVersion>,
            T::Error: ::std::fmt::Display,
        {
            self.protocol_version = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for protocol_version: {e}"));
            self
        }
        pub fn request_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<()>,
            T::Error: ::std::fmt::Display,
        {
            self.request_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for request_id: {e}"));
            self
        }
        pub fn session_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<()>,
            T::Error: ::std::fmt::Display,
        {
            self.session_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for session_id: {e}"));
            self
        }
        pub fn type_<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TransferResultMessageType>,
            T::Error: ::std::fmt::Display,
        {
            self.type_ = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for type_: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<TransferResultMessage> for super::TransferResultMessage {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TransferResultMessage,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                device_id: value.device_id?,
                payload: value.payload?,
                protocol_version: value.protocol_version?,
                request_id: value.request_id?,
                session_id: value.session_id?,
                type_: value.type_?,
            })
        }
    }
    impl ::std::convert::From<super::TransferResultMessage> for TransferResultMessage {
        fn from(value: super::TransferResultMessage) -> Self {
            Self {
                device_id: Ok(value.device_id),
                payload: Ok(value.payload),
                protocol_version: Ok(value.protocol_version),
                request_id: Ok(value.request_id),
                session_id: Ok(value.session_id),
                type_: Ok(value.type_),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TransferResultMessagePayload {
        code: ::std::result::Result<::std::option::Option<super::ErrorCode>, ::std::string::String>,
        message: ::std::result::Result<
            super::TransferResultMessagePayloadMessage,
            ::std::string::String,
        >,
        success: ::std::result::Result<bool, ::std::string::String>,
        transfer_id: ::std::result::Result<super::Uuid, ::std::string::String>,
    }
    impl ::std::default::Default for TransferResultMessagePayload {
        fn default() -> Self {
            Self {
                code: Err("no value supplied for code".to_string()),
                message: Err("no value supplied for message".to_string()),
                success: Err("no value supplied for success".to_string()),
                transfer_id: Err("no value supplied for transfer_id".to_string()),
            }
        }
    }
    impl TransferResultMessagePayload {
        pub fn code<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::option::Option<super::ErrorCode>>,
            T::Error: ::std::fmt::Display,
        {
            self.code = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for code: {e}"));
            self
        }
        pub fn message<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TransferResultMessagePayloadMessage>,
            T::Error: ::std::fmt::Display,
        {
            self.message = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for message: {e}"));
            self
        }
        pub fn success<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<bool>,
            T::Error: ::std::fmt::Display,
        {
            self.success = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for success: {e}"));
            self
        }
        pub fn transfer_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.transfer_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for transfer_id: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<TransferResultMessagePayload> for super::TransferResultMessagePayload {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TransferResultMessagePayload,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                code: value.code?,
                message: value.message?,
                success: value.success?,
                transfer_id: value.transfer_id?,
            })
        }
    }
    impl ::std::convert::From<super::TransferResultMessagePayload> for TransferResultMessagePayload {
        fn from(value: super::TransferResultMessagePayload) -> Self {
            Self {
                code: Ok(value.code),
                message: Ok(value.message),
                success: Ok(value.success),
                transfer_id: Ok(value.transfer_id),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TransferStartMessage {
        device_id: ::std::result::Result<super::Uuid, ::std::string::String>,
        payload: ::std::result::Result<super::TransferStartMessagePayload, ::std::string::String>,
        protocol_version: ::std::result::Result<
            super::TransferStartMessageProtocolVersion,
            ::std::string::String,
        >,
        request_id: ::std::result::Result<super::Uuid, ::std::string::String>,
        session_id: ::std::result::Result<(), ::std::string::String>,
        type_: ::std::result::Result<super::TransferStartMessageType, ::std::string::String>,
    }
    impl ::std::default::Default for TransferStartMessage {
        fn default() -> Self {
            Self {
                device_id: Err("no value supplied for device_id".to_string()),
                payload: Err("no value supplied for payload".to_string()),
                protocol_version: Err("no value supplied for protocol_version".to_string()),
                request_id: Err("no value supplied for request_id".to_string()),
                session_id: Err("no value supplied for session_id".to_string()),
                type_: Err("no value supplied for type_".to_string()),
            }
        }
    }
    impl TransferStartMessage {
        pub fn device_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.device_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for device_id: {e}"));
            self
        }
        pub fn payload<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TransferStartMessagePayload>,
            T::Error: ::std::fmt::Display,
        {
            self.payload = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for payload: {e}"));
            self
        }
        pub fn protocol_version<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TransferStartMessageProtocolVersion>,
            T::Error: ::std::fmt::Display,
        {
            self.protocol_version = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for protocol_version: {e}"));
            self
        }
        pub fn request_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.request_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for request_id: {e}"));
            self
        }
        pub fn session_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<()>,
            T::Error: ::std::fmt::Display,
        {
            self.session_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for session_id: {e}"));
            self
        }
        pub fn type_<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::TransferStartMessageType>,
            T::Error: ::std::fmt::Display,
        {
            self.type_ = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for type_: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<TransferStartMessage> for super::TransferStartMessage {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TransferStartMessage,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                device_id: value.device_id?,
                payload: value.payload?,
                protocol_version: value.protocol_version?,
                request_id: value.request_id?,
                session_id: value.session_id?,
                type_: value.type_?,
            })
        }
    }
    impl ::std::convert::From<super::TransferStartMessage> for TransferStartMessage {
        fn from(value: super::TransferStartMessage) -> Self {
            Self {
                device_id: Ok(value.device_id),
                payload: Ok(value.payload),
                protocol_version: Ok(value.protocol_version),
                request_id: Ok(value.request_id),
                session_id: Ok(value.session_id),
                type_: Ok(value.type_),
            }
        }
    }
    #[derive(Clone, Debug)]
    pub struct TransferStartMessagePayload {
        entries: ::std::result::Result<::std::vec::Vec<super::FileEntry>, ::std::string::String>,
        root_note: ::std::result::Result<super::SafeRelativePath, ::std::string::String>,
        transfer_id: ::std::result::Result<super::Uuid, ::std::string::String>,
    }
    impl ::std::default::Default for TransferStartMessagePayload {
        fn default() -> Self {
            Self {
                entries: Err("no value supplied for entries".to_string()),
                root_note: Err("no value supplied for root_note".to_string()),
                transfer_id: Err("no value supplied for transfer_id".to_string()),
            }
        }
    }
    impl TransferStartMessagePayload {
        pub fn entries<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<::std::vec::Vec<super::FileEntry>>,
            T::Error: ::std::fmt::Display,
        {
            self.entries = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for entries: {e}"));
            self
        }
        pub fn root_note<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::SafeRelativePath>,
            T::Error: ::std::fmt::Display,
        {
            self.root_note = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for root_note: {e}"));
            self
        }
        pub fn transfer_id<T>(mut self, value: T) -> Self
        where
            T: ::std::convert::TryInto<super::Uuid>,
            T::Error: ::std::fmt::Display,
        {
            self.transfer_id = value
                .try_into()
                .map_err(|e| format!("error converting supplied value for transfer_id: {e}"));
            self
        }
    }
    impl ::std::convert::TryFrom<TransferStartMessagePayload> for super::TransferStartMessagePayload {
        type Error = super::error::ConversionError;
        fn try_from(
            value: TransferStartMessagePayload,
        ) -> ::std::result::Result<Self, super::error::ConversionError> {
            Ok(Self {
                entries: value.entries?,
                root_note: value.root_note?,
                transfer_id: value.transfer_id?,
            })
        }
    }
    impl ::std::convert::From<super::TransferStartMessagePayload> for TransferStartMessagePayload {
        fn from(value: super::TransferStartMessagePayload) -> Self {
            Self {
                entries: Ok(value.entries),
                root_note: Ok(value.root_note),
                transfer_id: Ok(value.transfer_id),
            }
        }
    }
}
