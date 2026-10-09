// @generated
impl serde::Serialize for GetVibeRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("examples.v1.GetVibeRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetVibeRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                            Err(serde::de::Error::unknown_field(value, FIELDS))
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetVibeRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct examples.v1.GetVibeRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GetVibeRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map_.next_key::<GeneratedField>()?.is_some() {
                    let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(GetVibeRequest {
                })
            }
        }
        deserializer.deserialize_struct("examples.v1.GetVibeRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetVibeResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.vibe.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("examples.v1.GetVibeResponse", len)?;
        if !self.vibe.is_empty() {
            struct_ser.serialize_field("vibe", &self.vibe)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetVibeResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "vibe",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Vibe,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "vibe" => Ok(GeneratedField::Vibe),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetVibeResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct examples.v1.GetVibeResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GetVibeResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut vibe__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Vibe => {
                            if vibe__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibe"));
                            }
                            vibe__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(GetVibeResponse {
                    vibe: vibe__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("examples.v1.GetVibeResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SetVibeArrayRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.vibe_array.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("examples.v1.SetVibeArrayRequest", len)?;
        if let Some(v) = self.vibe_array.as_ref() {
            struct_ser.serialize_field("vibeArray", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SetVibeArrayRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "vibe_array",
            "vibeArray",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            VibeArray,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "vibeArray" | "vibe_array" => Ok(GeneratedField::VibeArray),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SetVibeArrayRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct examples.v1.SetVibeArrayRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SetVibeArrayRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut vibe_array__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::VibeArray => {
                            if vibe_array__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeArray"));
                            }
                            vibe_array__ = map_.next_value()?;
                        }
                    }
                }
                Ok(SetVibeArrayRequest {
                    vibe_array: vibe_array__,
                })
            }
        }
        deserializer.deserialize_struct("examples.v1.SetVibeArrayRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SetVibeArrayResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.vibe_array.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("examples.v1.SetVibeArrayResponse", len)?;
        if let Some(v) = self.vibe_array.as_ref() {
            struct_ser.serialize_field("vibeArray", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SetVibeArrayResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "vibe_array",
            "vibeArray",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            VibeArray,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "vibeArray" | "vibe_array" => Ok(GeneratedField::VibeArray),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SetVibeArrayResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct examples.v1.SetVibeArrayResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SetVibeArrayResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut vibe_array__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::VibeArray => {
                            if vibe_array__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeArray"));
                            }
                            vibe_array__ = map_.next_value()?;
                        }
                    }
                }
                Ok(SetVibeArrayResponse {
                    vibe_array: vibe_array__,
                })
            }
        }
        deserializer.deserialize_struct("examples.v1.SetVibeArrayResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SetVibeDetailsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.vibe.is_empty() {
            len += 1;
        }
        if self.vibe_scalar.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("examples.v1.SetVibeDetailsRequest", len)?;
        if !self.vibe.is_empty() {
            struct_ser.serialize_field("vibe", &self.vibe)?;
        }
        if let Some(v) = self.vibe_scalar.as_ref() {
            struct_ser.serialize_field("vibeScalar", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SetVibeDetailsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "vibe",
            "vibe_scalar",
            "vibeScalar",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Vibe,
            VibeScalar,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "vibe" => Ok(GeneratedField::Vibe),
                            "vibeScalar" | "vibe_scalar" => Ok(GeneratedField::VibeScalar),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SetVibeDetailsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct examples.v1.SetVibeDetailsRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SetVibeDetailsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut vibe__ = None;
                let mut vibe_scalar__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Vibe => {
                            if vibe__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibe"));
                            }
                            vibe__ = Some(map_.next_value()?);
                        }
                        GeneratedField::VibeScalar => {
                            if vibe_scalar__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeScalar"));
                            }
                            vibe_scalar__ = map_.next_value()?;
                        }
                    }
                }
                Ok(SetVibeDetailsRequest {
                    vibe: vibe__.unwrap_or_default(),
                    vibe_scalar: vibe_scalar__,
                })
            }
        }
        deserializer.deserialize_struct("examples.v1.SetVibeDetailsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SetVibeObjectsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.vibe_object.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("examples.v1.SetVibeObjectsRequest", len)?;
        if !self.vibe_object.is_empty() {
            struct_ser.serialize_field("vibeObject", &self.vibe_object)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SetVibeObjectsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "vibe_object",
            "vibeObject",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            VibeObject,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "vibeObject" | "vibe_object" => Ok(GeneratedField::VibeObject),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SetVibeObjectsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct examples.v1.SetVibeObjectsRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SetVibeObjectsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut vibe_object__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::VibeObject => {
                            if vibe_object__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeObject"));
                            }
                            vibe_object__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(SetVibeObjectsRequest {
                    vibe_object: vibe_object__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("examples.v1.SetVibeObjectsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SetVibeObjectsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.vibe_object.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("examples.v1.SetVibeObjectsResponse", len)?;
        if !self.vibe_object.is_empty() {
            struct_ser.serialize_field("vibeObject", &self.vibe_object)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SetVibeObjectsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "vibe_object",
            "vibeObject",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            VibeObject,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "vibeObject" | "vibe_object" => Ok(GeneratedField::VibeObject),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SetVibeObjectsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct examples.v1.SetVibeObjectsResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SetVibeObjectsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut vibe_object__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::VibeObject => {
                            if vibe_object__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeObject"));
                            }
                            vibe_object__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(SetVibeObjectsResponse {
                    vibe_object: vibe_object__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("examples.v1.SetVibeObjectsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SetVibeRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.vibe.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("examples.v1.SetVibeRequest", len)?;
        if !self.vibe.is_empty() {
            struct_ser.serialize_field("vibe", &self.vibe)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SetVibeRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "vibe",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Vibe,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "vibe" => Ok(GeneratedField::Vibe),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SetVibeRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct examples.v1.SetVibeRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SetVibeRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut vibe__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Vibe => {
                            if vibe__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibe"));
                            }
                            vibe__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(SetVibeRequest {
                    vibe: vibe__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("examples.v1.SetVibeRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SetVibeResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.previous_vibe.is_empty() {
            len += 1;
        }
        if !self.vibe.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("examples.v1.SetVibeResponse", len)?;
        if !self.previous_vibe.is_empty() {
            struct_ser.serialize_field("previousVibe", &self.previous_vibe)?;
        }
        if !self.vibe.is_empty() {
            struct_ser.serialize_field("vibe", &self.vibe)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SetVibeResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "previous_vibe",
            "previousVibe",
            "vibe",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PreviousVibe,
            Vibe,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "previousVibe" | "previous_vibe" => Ok(GeneratedField::PreviousVibe),
                            "vibe" => Ok(GeneratedField::Vibe),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SetVibeResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct examples.v1.SetVibeResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SetVibeResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut previous_vibe__ = None;
                let mut vibe__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PreviousVibe => {
                            if previous_vibe__.is_some() {
                                return Err(serde::de::Error::duplicate_field("previousVibe"));
                            }
                            previous_vibe__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Vibe => {
                            if vibe__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibe"));
                            }
                            vibe__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(SetVibeResponse {
                    previous_vibe: previous_vibe__.unwrap_or_default(),
                    vibe: vibe__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("examples.v1.SetVibeResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SomeVibeObject {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.vibe.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("examples.v1.SomeVibeObject", len)?;
        if !self.vibe.is_empty() {
            struct_ser.serialize_field("vibe", &self.vibe)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SomeVibeObject {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "vibe",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Vibe,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "vibe" => Ok(GeneratedField::Vibe),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SomeVibeObject;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct examples.v1.SomeVibeObject")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SomeVibeObject, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut vibe__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Vibe => {
                            if vibe__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibe"));
                            }
                            vibe__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(SomeVibeObject {
                    vibe: vibe__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("examples.v1.SomeVibeObject", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for VibeArray {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.vibe_doubles.is_empty() {
            len += 1;
        }
        if !self.vibe_floats.is_empty() {
            len += 1;
        }
        if !self.vibe_int32s.is_empty() {
            len += 1;
        }
        if !self.vibe_int64s.is_empty() {
            len += 1;
        }
        if !self.vibe_uint32s.is_empty() {
            len += 1;
        }
        if !self.vibe_uint64s.is_empty() {
            len += 1;
        }
        if !self.vibe_sint32s.is_empty() {
            len += 1;
        }
        if !self.vibe_sint64s.is_empty() {
            len += 1;
        }
        if !self.vibe_fixed32s.is_empty() {
            len += 1;
        }
        if !self.vibe_fixed64s.is_empty() {
            len += 1;
        }
        if !self.vibe_sfixed32s.is_empty() {
            len += 1;
        }
        if !self.vibe_sfixed64s.is_empty() {
            len += 1;
        }
        if !self.vibe_bools.is_empty() {
            len += 1;
        }
        if !self.vibe_byteses.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("examples.v1.VibeArray", len)?;
        if !self.vibe_doubles.is_empty() {
            struct_ser.serialize_field("vibeDoubles", &self.vibe_doubles)?;
        }
        if !self.vibe_floats.is_empty() {
            struct_ser.serialize_field("vibeFloats", &self.vibe_floats)?;
        }
        if !self.vibe_int32s.is_empty() {
            struct_ser.serialize_field("vibeInt32s", &self.vibe_int32s)?;
        }
        if !self.vibe_int64s.is_empty() {
            struct_ser.serialize_field("vibeInt64s", &self.vibe_int64s.iter().map(ToString::to_string).collect::<Vec<_>>())?;
        }
        if !self.vibe_uint32s.is_empty() {
            struct_ser.serialize_field("vibeUint32s", &self.vibe_uint32s)?;
        }
        if !self.vibe_uint64s.is_empty() {
            struct_ser.serialize_field("vibeUint64s", &self.vibe_uint64s.iter().map(ToString::to_string).collect::<Vec<_>>())?;
        }
        if !self.vibe_sint32s.is_empty() {
            struct_ser.serialize_field("vibeSint32s", &self.vibe_sint32s)?;
        }
        if !self.vibe_sint64s.is_empty() {
            struct_ser.serialize_field("vibeSint64s", &self.vibe_sint64s.iter().map(ToString::to_string).collect::<Vec<_>>())?;
        }
        if !self.vibe_fixed32s.is_empty() {
            struct_ser.serialize_field("vibeFixed32s", &self.vibe_fixed32s)?;
        }
        if !self.vibe_fixed64s.is_empty() {
            struct_ser.serialize_field("vibeFixed64s", &self.vibe_fixed64s.iter().map(ToString::to_string).collect::<Vec<_>>())?;
        }
        if !self.vibe_sfixed32s.is_empty() {
            struct_ser.serialize_field("vibeSfixed32s", &self.vibe_sfixed32s)?;
        }
        if !self.vibe_sfixed64s.is_empty() {
            struct_ser.serialize_field("vibeSfixed64s", &self.vibe_sfixed64s.iter().map(ToString::to_string).collect::<Vec<_>>())?;
        }
        if !self.vibe_bools.is_empty() {
            struct_ser.serialize_field("vibeBools", &self.vibe_bools)?;
        }
        if !self.vibe_byteses.is_empty() {
            struct_ser.serialize_field("vibeByteses", &self.vibe_byteses.iter().map(pbjson::private::base64::encode).collect::<Vec<_>>())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for VibeArray {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "vibe_doubles",
            "vibeDoubles",
            "vibe_floats",
            "vibeFloats",
            "vibe_int32s",
            "vibeInt32s",
            "vibe_int64s",
            "vibeInt64s",
            "vibe_uint32s",
            "vibeUint32s",
            "vibe_uint64s",
            "vibeUint64s",
            "vibe_sint32s",
            "vibeSint32s",
            "vibe_sint64s",
            "vibeSint64s",
            "vibe_fixed32s",
            "vibeFixed32s",
            "vibe_fixed64s",
            "vibeFixed64s",
            "vibe_sfixed32s",
            "vibeSfixed32s",
            "vibe_sfixed64s",
            "vibeSfixed64s",
            "vibe_bools",
            "vibeBools",
            "vibe_byteses",
            "vibeByteses",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            VibeDoubles,
            VibeFloats,
            VibeInt32s,
            VibeInt64s,
            VibeUint32s,
            VibeUint64s,
            VibeSint32s,
            VibeSint64s,
            VibeFixed32s,
            VibeFixed64s,
            VibeSfixed32s,
            VibeSfixed64s,
            VibeBools,
            VibeByteses,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "vibeDoubles" | "vibe_doubles" => Ok(GeneratedField::VibeDoubles),
                            "vibeFloats" | "vibe_floats" => Ok(GeneratedField::VibeFloats),
                            "vibeInt32s" | "vibe_int32s" => Ok(GeneratedField::VibeInt32s),
                            "vibeInt64s" | "vibe_int64s" => Ok(GeneratedField::VibeInt64s),
                            "vibeUint32s" | "vibe_uint32s" => Ok(GeneratedField::VibeUint32s),
                            "vibeUint64s" | "vibe_uint64s" => Ok(GeneratedField::VibeUint64s),
                            "vibeSint32s" | "vibe_sint32s" => Ok(GeneratedField::VibeSint32s),
                            "vibeSint64s" | "vibe_sint64s" => Ok(GeneratedField::VibeSint64s),
                            "vibeFixed32s" | "vibe_fixed32s" => Ok(GeneratedField::VibeFixed32s),
                            "vibeFixed64s" | "vibe_fixed64s" => Ok(GeneratedField::VibeFixed64s),
                            "vibeSfixed32s" | "vibe_sfixed32s" => Ok(GeneratedField::VibeSfixed32s),
                            "vibeSfixed64s" | "vibe_sfixed64s" => Ok(GeneratedField::VibeSfixed64s),
                            "vibeBools" | "vibe_bools" => Ok(GeneratedField::VibeBools),
                            "vibeByteses" | "vibe_byteses" => Ok(GeneratedField::VibeByteses),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = VibeArray;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct examples.v1.VibeArray")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<VibeArray, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut vibe_doubles__ = None;
                let mut vibe_floats__ = None;
                let mut vibe_int32s__ = None;
                let mut vibe_int64s__ = None;
                let mut vibe_uint32s__ = None;
                let mut vibe_uint64s__ = None;
                let mut vibe_sint32s__ = None;
                let mut vibe_sint64s__ = None;
                let mut vibe_fixed32s__ = None;
                let mut vibe_fixed64s__ = None;
                let mut vibe_sfixed32s__ = None;
                let mut vibe_sfixed64s__ = None;
                let mut vibe_bools__ = None;
                let mut vibe_byteses__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::VibeDoubles => {
                            if vibe_doubles__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeDoubles"));
                            }
                            vibe_doubles__ = 
                                Some(map_.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                        GeneratedField::VibeFloats => {
                            if vibe_floats__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeFloats"));
                            }
                            vibe_floats__ = 
                                Some(map_.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                        GeneratedField::VibeInt32s => {
                            if vibe_int32s__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeInt32s"));
                            }
                            vibe_int32s__ = 
                                Some(map_.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                        GeneratedField::VibeInt64s => {
                            if vibe_int64s__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeInt64s"));
                            }
                            vibe_int64s__ = 
                                Some(map_.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                        GeneratedField::VibeUint32s => {
                            if vibe_uint32s__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeUint32s"));
                            }
                            vibe_uint32s__ = 
                                Some(map_.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                        GeneratedField::VibeUint64s => {
                            if vibe_uint64s__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeUint64s"));
                            }
                            vibe_uint64s__ = 
                                Some(map_.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                        GeneratedField::VibeSint32s => {
                            if vibe_sint32s__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeSint32s"));
                            }
                            vibe_sint32s__ = 
                                Some(map_.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                        GeneratedField::VibeSint64s => {
                            if vibe_sint64s__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeSint64s"));
                            }
                            vibe_sint64s__ = 
                                Some(map_.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                        GeneratedField::VibeFixed32s => {
                            if vibe_fixed32s__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeFixed32s"));
                            }
                            vibe_fixed32s__ = 
                                Some(map_.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                        GeneratedField::VibeFixed64s => {
                            if vibe_fixed64s__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeFixed64s"));
                            }
                            vibe_fixed64s__ = 
                                Some(map_.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                        GeneratedField::VibeSfixed32s => {
                            if vibe_sfixed32s__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeSfixed32s"));
                            }
                            vibe_sfixed32s__ = 
                                Some(map_.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                        GeneratedField::VibeSfixed64s => {
                            if vibe_sfixed64s__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeSfixed64s"));
                            }
                            vibe_sfixed64s__ = 
                                Some(map_.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                        GeneratedField::VibeBools => {
                            if vibe_bools__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeBools"));
                            }
                            vibe_bools__ = Some(map_.next_value()?);
                        }
                        GeneratedField::VibeByteses => {
                            if vibe_byteses__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeByteses"));
                            }
                            vibe_byteses__ = 
                                Some(map_.next_value::<Vec<::pbjson::private::BytesDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                    }
                }
                Ok(VibeArray {
                    vibe_doubles: vibe_doubles__.unwrap_or_default(),
                    vibe_floats: vibe_floats__.unwrap_or_default(),
                    vibe_int32s: vibe_int32s__.unwrap_or_default(),
                    vibe_int64s: vibe_int64s__.unwrap_or_default(),
                    vibe_uint32s: vibe_uint32s__.unwrap_or_default(),
                    vibe_uint64s: vibe_uint64s__.unwrap_or_default(),
                    vibe_sint32s: vibe_sint32s__.unwrap_or_default(),
                    vibe_sint64s: vibe_sint64s__.unwrap_or_default(),
                    vibe_fixed32s: vibe_fixed32s__.unwrap_or_default(),
                    vibe_fixed64s: vibe_fixed64s__.unwrap_or_default(),
                    vibe_sfixed32s: vibe_sfixed32s__.unwrap_or_default(),
                    vibe_sfixed64s: vibe_sfixed64s__.unwrap_or_default(),
                    vibe_bools: vibe_bools__.unwrap_or_default(),
                    vibe_byteses: vibe_byteses__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("examples.v1.VibeArray", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for VibeScalar {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.vibe_double != 0. {
            len += 1;
        }
        if self.vibe_float != 0. {
            len += 1;
        }
        if self.vibe_int32 != 0 {
            len += 1;
        }
        if self.vibe_int64 != 0 {
            len += 1;
        }
        if self.vibe_uint32.is_some() {
            len += 1;
        }
        if self.vibe_uint64 != 0 {
            len += 1;
        }
        if self.vibe_sint32 != 0 {
            len += 1;
        }
        if self.vibe_sint64 != 0 {
            len += 1;
        }
        if self.vibe_fixed32 != 0 {
            len += 1;
        }
        if self.vibe_fixed64 != 0 {
            len += 1;
        }
        if self.vibe_sfixed32 != 0 {
            len += 1;
        }
        if self.vibe_sfixed64 != 0 {
            len += 1;
        }
        if self.vibe_bool {
            len += 1;
        }
        if !self.vibe_bytes.is_empty() {
            len += 1;
        }
        if !self.vibe_enum.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("examples.v1.VibeScalar", len)?;
        if self.vibe_double != 0. {
            struct_ser.serialize_field("vibeDouble", &self.vibe_double)?;
        }
        if self.vibe_float != 0. {
            struct_ser.serialize_field("vibeFloat", &self.vibe_float)?;
        }
        if self.vibe_int32 != 0 {
            struct_ser.serialize_field("vibeInt32", &self.vibe_int32)?;
        }
        if self.vibe_int64 != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("vibeInt64", ToString::to_string(&self.vibe_int64).as_str())?;
        }
        if let Some(v) = self.vibe_uint32.as_ref() {
            struct_ser.serialize_field("vibeUint32", v)?;
        }
        if self.vibe_uint64 != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("vibeUint64", ToString::to_string(&self.vibe_uint64).as_str())?;
        }
        if self.vibe_sint32 != 0 {
            struct_ser.serialize_field("vibeSint32", &self.vibe_sint32)?;
        }
        if self.vibe_sint64 != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("vibeSint64", ToString::to_string(&self.vibe_sint64).as_str())?;
        }
        if self.vibe_fixed32 != 0 {
            struct_ser.serialize_field("vibeFixed32", &self.vibe_fixed32)?;
        }
        if self.vibe_fixed64 != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("vibeFixed64", ToString::to_string(&self.vibe_fixed64).as_str())?;
        }
        if self.vibe_sfixed32 != 0 {
            struct_ser.serialize_field("vibeSfixed32", &self.vibe_sfixed32)?;
        }
        if self.vibe_sfixed64 != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("vibeSfixed64", ToString::to_string(&self.vibe_sfixed64).as_str())?;
        }
        if self.vibe_bool {
            struct_ser.serialize_field("vibeBool", &self.vibe_bool)?;
        }
        if !self.vibe_bytes.is_empty() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("vibeBytes", pbjson::private::base64::encode(&self.vibe_bytes).as_str())?;
        }
        if !self.vibe_enum.is_empty() {
            let v = self.vibe_enum.iter().cloned().map(|v| {
                vibe_scalar::VibeEnum::try_from(v)
                    .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", v)))
                }).collect::<std::result::Result<Vec<_>, _>>()?;
            struct_ser.serialize_field("vibeEnum", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for VibeScalar {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "vibe_double",
            "vibeDouble",
            "vibe_float",
            "vibeFloat",
            "vibe_int32",
            "vibeInt32",
            "vibe_int64",
            "vibeInt64",
            "vibe_uint32",
            "vibeUint32",
            "vibe_uint64",
            "vibeUint64",
            "vibe_sint32",
            "vibeSint32",
            "vibe_sint64",
            "vibeSint64",
            "vibe_fixed32",
            "vibeFixed32",
            "vibe_fixed64",
            "vibeFixed64",
            "vibe_sfixed32",
            "vibeSfixed32",
            "vibe_sfixed64",
            "vibeSfixed64",
            "vibe_bool",
            "vibeBool",
            "vibe_bytes",
            "vibeBytes",
            "vibe_enum",
            "vibeEnum",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            VibeDouble,
            VibeFloat,
            VibeInt32,
            VibeInt64,
            VibeUint32,
            VibeUint64,
            VibeSint32,
            VibeSint64,
            VibeFixed32,
            VibeFixed64,
            VibeSfixed32,
            VibeSfixed64,
            VibeBool,
            VibeBytes,
            VibeEnum,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "vibeDouble" | "vibe_double" => Ok(GeneratedField::VibeDouble),
                            "vibeFloat" | "vibe_float" => Ok(GeneratedField::VibeFloat),
                            "vibeInt32" | "vibe_int32" => Ok(GeneratedField::VibeInt32),
                            "vibeInt64" | "vibe_int64" => Ok(GeneratedField::VibeInt64),
                            "vibeUint32" | "vibe_uint32" => Ok(GeneratedField::VibeUint32),
                            "vibeUint64" | "vibe_uint64" => Ok(GeneratedField::VibeUint64),
                            "vibeSint32" | "vibe_sint32" => Ok(GeneratedField::VibeSint32),
                            "vibeSint64" | "vibe_sint64" => Ok(GeneratedField::VibeSint64),
                            "vibeFixed32" | "vibe_fixed32" => Ok(GeneratedField::VibeFixed32),
                            "vibeFixed64" | "vibe_fixed64" => Ok(GeneratedField::VibeFixed64),
                            "vibeSfixed32" | "vibe_sfixed32" => Ok(GeneratedField::VibeSfixed32),
                            "vibeSfixed64" | "vibe_sfixed64" => Ok(GeneratedField::VibeSfixed64),
                            "vibeBool" | "vibe_bool" => Ok(GeneratedField::VibeBool),
                            "vibeBytes" | "vibe_bytes" => Ok(GeneratedField::VibeBytes),
                            "vibeEnum" | "vibe_enum" => Ok(GeneratedField::VibeEnum),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = VibeScalar;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct examples.v1.VibeScalar")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<VibeScalar, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut vibe_double__ = None;
                let mut vibe_float__ = None;
                let mut vibe_int32__ = None;
                let mut vibe_int64__ = None;
                let mut vibe_uint32__ = None;
                let mut vibe_uint64__ = None;
                let mut vibe_sint32__ = None;
                let mut vibe_sint64__ = None;
                let mut vibe_fixed32__ = None;
                let mut vibe_fixed64__ = None;
                let mut vibe_sfixed32__ = None;
                let mut vibe_sfixed64__ = None;
                let mut vibe_bool__ = None;
                let mut vibe_bytes__ = None;
                let mut vibe_enum__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::VibeDouble => {
                            if vibe_double__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeDouble"));
                            }
                            vibe_double__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::VibeFloat => {
                            if vibe_float__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeFloat"));
                            }
                            vibe_float__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::VibeInt32 => {
                            if vibe_int32__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeInt32"));
                            }
                            vibe_int32__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::VibeInt64 => {
                            if vibe_int64__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeInt64"));
                            }
                            vibe_int64__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::VibeUint32 => {
                            if vibe_uint32__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeUint32"));
                            }
                            vibe_uint32__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::VibeUint64 => {
                            if vibe_uint64__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeUint64"));
                            }
                            vibe_uint64__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::VibeSint32 => {
                            if vibe_sint32__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeSint32"));
                            }
                            vibe_sint32__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::VibeSint64 => {
                            if vibe_sint64__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeSint64"));
                            }
                            vibe_sint64__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::VibeFixed32 => {
                            if vibe_fixed32__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeFixed32"));
                            }
                            vibe_fixed32__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::VibeFixed64 => {
                            if vibe_fixed64__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeFixed64"));
                            }
                            vibe_fixed64__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::VibeSfixed32 => {
                            if vibe_sfixed32__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeSfixed32"));
                            }
                            vibe_sfixed32__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::VibeSfixed64 => {
                            if vibe_sfixed64__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeSfixed64"));
                            }
                            vibe_sfixed64__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::VibeBool => {
                            if vibe_bool__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeBool"));
                            }
                            vibe_bool__ = Some(map_.next_value()?);
                        }
                        GeneratedField::VibeBytes => {
                            if vibe_bytes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeBytes"));
                            }
                            vibe_bytes__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::VibeEnum => {
                            if vibe_enum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vibeEnum"));
                            }
                            vibe_enum__ = Some(map_.next_value::<Vec<vibe_scalar::VibeEnum>>()?.into_iter().map(|x| x as i32).collect());
                        }
                    }
                }
                Ok(VibeScalar {
                    vibe_double: vibe_double__.unwrap_or_default(),
                    vibe_float: vibe_float__.unwrap_or_default(),
                    vibe_int32: vibe_int32__.unwrap_or_default(),
                    vibe_int64: vibe_int64__.unwrap_or_default(),
                    vibe_uint32: vibe_uint32__,
                    vibe_uint64: vibe_uint64__.unwrap_or_default(),
                    vibe_sint32: vibe_sint32__.unwrap_or_default(),
                    vibe_sint64: vibe_sint64__.unwrap_or_default(),
                    vibe_fixed32: vibe_fixed32__.unwrap_or_default(),
                    vibe_fixed64: vibe_fixed64__.unwrap_or_default(),
                    vibe_sfixed32: vibe_sfixed32__.unwrap_or_default(),
                    vibe_sfixed64: vibe_sfixed64__.unwrap_or_default(),
                    vibe_bool: vibe_bool__.unwrap_or_default(),
                    vibe_bytes: vibe_bytes__.unwrap_or_default(),
                    vibe_enum: vibe_enum__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("examples.v1.VibeScalar", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for vibe_scalar::VibeEnum {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::VibeUnset => "VIBE_UNSET",
            Self::VibeGood => "VIBE_GOOD",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for vibe_scalar::VibeEnum {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "VIBE_UNSET",
            "VIBE_GOOD",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = vibe_scalar::VibeEnum;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "VIBE_UNSET" => Ok(vibe_scalar::VibeEnum::VibeUnset),
                    "VIBE_GOOD" => Ok(vibe_scalar::VibeEnum::VibeGood),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
