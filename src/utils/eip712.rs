use std::collections::HashMap;

use alloy::{
    dyn_abi::{DynSolType, DynSolValue},
    hex,
    primitives::{Address, I256, U256},
};
use serde_json::Value;

pub fn eip712_to_dyn(
    td: &Value,
) -> crate::Result<(DynSolType, DynSolValue, DynSolType, DynSolValue)> {
    let types = td
        .get("types")
        .ok_or(crate::Error::InternalErrorStr("missing types"))?;
    let domain = td
        .get("domain")
        .ok_or(crate::Error::InternalErrorStr("missing domain"))?;
    let primary = td
        .get("primaryType")
        .and_then(|v| v.as_str())
        .ok_or(crate::Error::InternalErrorStr("missing primaryType"))?;
    let message = td
        .get("message")
        .ok_or(crate::Error::InternalErrorStr("missing message"))?;

    let env = TypeEnv::build(types)?;

    let (domain_ty, domain_val) = env.build_struct_and_value("EIP712Domain", domain)?;
    let (msg_ty, msg_val) = env.build_struct_and_value(primary, message)?;

    Ok((msg_ty, msg_val, domain_ty, domain_val))
}

#[derive(Clone)]
struct StructSpec {
    field_names: Vec<String>,
    field_types: Vec<DynSolType>,
}

struct TypeEnv {
    structs: HashMap<String, (DynSolType, StructSpec)>,
}

impl TypeEnv {
    fn build(types: &Value) -> crate::Result<Self> {
        let obj = types
            .as_object()
            .ok_or(crate::Error::InternalErrorStr("types must be an object"))?;

        let mut structs: HashMap<String, (DynSolType, StructSpec)> = HashMap::new();

        for (name, fields_v) in obj {
            let fields = fields_v
                .as_array()
                .ok_or(crate::Error::InternalError(format!(
                    "types.{name} must be an array"
                )))?;
            let tuple = DynSolType::Tuple(vec![DynSolType::Bool; fields.len()]);
            let spec = StructSpec {
                field_names: vec!["".into(); fields.len()],
                field_types: vec![DynSolType::Bool; fields.len()],
            };
            structs.insert(name.clone(), (tuple, spec));
        }

        for (name, fields_v) in obj {
            let fields = fields_v
                .as_array()
                .ok_or(crate::Error::InternalError(format!(
                    "types.{name} must be an array"
                )))?;
            let mut names = Vec::with_capacity(fields.len());
            let mut types_vec = Vec::with_capacity(fields.len());

            for f in fields {
                let fname = f
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or(crate::Error::InternalError(format!(
                        "types.{name}.* missing name"
                    )))?
                    .to_string();

                let fty_str =
                    f.get("type")
                        .and_then(|v| v.as_str())
                        .ok_or(crate::Error::InternalError(format!(
                            "types.{name}.{fname} missing type"
                        )))?;

                let fty = resolve_type(fty_str, &structs)?;
                names.push(fname);
                types_vec.push(fty);
            }

            let tuple = DynSolType::Tuple(types_vec.clone());
            let spec = StructSpec {
                field_names: names,
                field_types: types_vec,
            };
            structs.insert(name.clone(), (tuple, spec));
        }

        Ok(Self { structs })
    }

    fn build_struct_and_value(
        &self,
        struct_name: &str,
        json_val: &Value,
    ) -> crate::Result<(DynSolType, DynSolValue)> {
        let (tuple_ty, spec) = self
            .structs
            .get(struct_name)
            .ok_or(crate::Error::InternalError(format!(
                "struct {struct_name} not found"
            )))?;

        let obj = json_val
            .as_object()
            .ok_or(crate::Error::InternalError(format!(
                "{struct_name} value must be an object"
            )))?;

        let mut vals = Vec::with_capacity(spec.field_names.len());
        for (fname, fty) in spec.field_names.iter().zip(spec.field_types.iter()) {
            let field_json = obj.get(fname).ok_or(crate::Error::InternalError(format!(
                "{struct_name}.{fname} missing"
            )))?;
            let v = self.build_value_from_type(fty, field_json)?;
            vals.push(v);
        }

        Ok((tuple_ty.clone(), DynSolValue::Tuple(vals)))
    }

    fn build_value_from_type(&self, ty: &DynSolType, v: &Value) -> crate::Result<DynSolValue> {
        Ok(match ty {
            DynSolType::Address => DynSolValue::Address(parse_address(v)?),
            DynSolType::Bool => DynSolValue::Bool(parse_bool(v)?),
            DynSolType::String => DynSolValue::String(
                v.as_str()
                    .ok_or(crate::Error::InternalErrorStr("expected string"))?
                    .to_string(),
            ),
            DynSolType::Bytes => DynSolValue::Bytes(parse_bytes(v)?),
            DynSolType::FixedBytes(n) => {
                DynSolValue::FixedBytes(parse_fixed_bytes(v, *n)?.into(), *n)
            }
            DynSolType::Uint(bits) => DynSolValue::Uint(parse_u256(v)?, *bits),
            DynSolType::Int(bits) => DynSolValue::Int(parse_i256(v)?, *bits),
            DynSolType::Array(inner) => {
                let arr = v
                    .as_array()
                    .ok_or(crate::Error::InternalErrorStr("expected array"))?;
                DynSolValue::Array(
                    arr.iter()
                        .map(|x| self.build_value_from_type(inner, x))
                        .collect::<crate::Result<Vec<_>>>()?,
                )
            }
            DynSolType::FixedArray(inner, n) => {
                let arr = v
                    .as_array()
                    .ok_or(crate::Error::InternalErrorStr("expected array"))?;
                if arr.len() != *n {
                    return Err(crate::Error::InternalError(format!(
                        "fixed array length mismatch: expected {n}, got {}",
                        arr.len()
                    )));
                }
                DynSolValue::FixedArray(
                    arr.iter()
                        .map(|x| self.build_value_from_type(inner, x))
                        .collect::<crate::Result<Vec<_>>>()?,
                )
            }
            DynSolType::Tuple(_field_tys) => {
                let obj = v.as_object().ok_or(crate::Error::InternalErrorStr(
                    "expected object for tuple/struct",
                ))?;

                let mut found: Option<&(DynSolType, StructSpec)> = None;
                for entry in self.structs.values() {
                    let (cand_ty, cand_spec) = entry;
                    if cand_ty == ty
                        && cand_spec
                            .field_names
                            .iter()
                            .all(|fname| obj.contains_key(fname))
                    {
                        found = Some(entry);
                        break;
                    }
                }
                let (_cand_ty, cand_spec) = found.ok_or(crate::Error::InternalErrorStr(
                    "cannot match inline struct to a named type",
                ))?;

                let mut vals = Vec::with_capacity(cand_spec.field_names.len());
                for (fname, fty) in cand_spec
                    .field_names
                    .iter()
                    .zip(cand_spec.field_types.iter())
                {
                    let field_json = obj.get(fname).ok_or(crate::Error::InternalError(format!(
                        "missing field {fname}"
                    )))?;
                    vals.push(self.build_value_from_type(fty, field_json)?);
                }
                DynSolValue::Tuple(vals)
            }
            other => {
                return Err(crate::Error::InternalError(format!(
                    "unsupported dyn type variant: {other:?}"
                )))
            }
        })
    }
}

fn parse_address(v: &Value) -> crate::Result<Address> {
    let s = v
        .as_str()
        .ok_or(crate::Error::InternalErrorStr("address must be string"))?;
    s.parse::<Address>()
        .map_err(|e| crate::Error::InternalError(format!("invalid address: {e}")))
}

fn parse_bool(v: &Value) -> crate::Result<bool> {
    if let Some(b) = v.as_bool() {
        return Ok(b);
    }
    if let Some(s) = v.as_str() {
        return Ok(match s {
            "true" | "1" => true,
            "false" | "0" => false,
            _ => {
                return Err(crate::Error::InternalError(format!(
                    "invalid bool string {s}"
                )))
            }
        });
    }
    Err(crate::Error::InternalErrorStr("invalid bool"))
}

fn parse_bytes(v: &Value) -> crate::Result<Vec<u8>> {
    let s = v
        .as_str()
        .ok_or(crate::Error::InternalErrorStr("bytes must be hex string"))?;
    let s = s.strip_prefix("0x").unwrap_or(s);
    hex::decode(s).map_err(|e| crate::Error::InternalError(format!("hex decode error: {e}")))
}

fn parse_fixed_bytes(v: &Value, n: usize) -> crate::Result<[u8; 32]> {
    if n > 32 {
        return Err(crate::Error::InternalError(format!(
            "bytes{n} exceeds max 32"
        )));
    }
    let s = v
        .as_str()
        .ok_or(crate::Error::InternalErrorStr("bytesN must be hex string"))?;
    let s = s.strip_prefix("0x").unwrap_or(s);
    let raw = hex::decode(s)
        .map_err(|e| crate::Error::InternalError(format!("hex decode error: {e}")))?;
    if raw.len() != n {
        return Err(crate::Error::InternalError(format!(
            "bytes{n} length mismatch: got {}",
            raw.len()
        )));
    }
    let mut out = [0u8; 32];
    out[..n].copy_from_slice(&raw);
    Ok(out)
}

fn parse_u256(v: &Value) -> crate::Result<U256> {
    if let Some(s) = v.as_str() {
        if let Ok(x) = U256::from_str_radix(s, 10) {
            return Ok(x);
        }
        let s2 = s.strip_prefix("0x").unwrap_or(s);
        let bytes = hex::decode(s2)
            .map_err(|e| crate::Error::InternalError(format!("hex decode error: {e}")))?;
        return Ok(U256::from_be_slice(&bytes));
    }
    if let Some(n) = v.as_u64() {
        return Ok(U256::from(n));
    }
    Err(crate::Error::InternalErrorStr("invalid uint"))
}

fn parse_i256(v: &serde_json::Value) -> crate::Result<I256> {
    if let Some(s) = v.as_str() {
        if let Some(hex) = s.strip_prefix("0x") {
            let bytes = alloy::hex::decode(hex)
                .map_err(|e| crate::Error::InternalError(format!("hex decode error: {e}")))?;
            let mag = U256::from_be_slice(&bytes);
            return Ok(I256::from_raw(mag));
        } else {
            let neg = s.starts_with('-');
            let digits = if neg { &s[1..] } else { s };
            let mag = U256::from_str_radix(digits, 10)
                .map_err(|e| crate::Error::InternalError(format!("invalid decimal int: {e}")))?;
            let i = I256::from_raw(mag);
            return Ok(if neg { -i } else { i });
        }
    }
    if let Some(n) = v.as_i64() {
        let mag = U256::from(n.unsigned_abs());
        let i = I256::from_raw(mag);
        return Ok(if n < 0 { -i } else { i });
    }
    Err(crate::Error::InternalErrorStr("invalid int"))
}

fn resolve_type(
    ty: &str,
    structs: &HashMap<String, (DynSolType, StructSpec)>,
) -> crate::Result<DynSolType> {
    if let Some((inner, len)) = parse_array_type(ty)? {
        let inner_ty = resolve_type(inner, structs)?;
        return Ok(match len {
            None => DynSolType::Array(Box::new(inner_ty)),
            Some(n) => DynSolType::FixedArray(Box::new(inner_ty), n),
        });
    }

    if let Some(n) = parse_bytes_n(ty)? {
        return Ok(DynSolType::FixedBytes(n));
    }

    if ty == "bytes" {
        return Ok(DynSolType::Bytes);
    }

    if ty == "string" {
        return Ok(DynSolType::String);
    }

    if ty == "address" {
        return Ok(DynSolType::Address);
    }

    if ty == "bool" {
        return Ok(DynSolType::Bool);
    }

    if let Some(bits) = parse_uint_bits(ty)? {
        return Ok(DynSolType::Uint(bits));
    }

    if let Some(bits) = parse_int_bits(ty)? {
        return Ok(DynSolType::Int(bits));
    }

    if let Some((t, _)) = structs.get(ty) {
        return Ok(t.clone());
    }

    Err(crate::Error::InternalError(format!("unknown type: {ty}")))
}

fn parse_uint_bits(s: &str) -> crate::Result<Option<usize>> {
    if s == "uint" {
        return Ok(Some(256));
    }
    if let Some(rest) = s.strip_prefix("uint") {
        if rest.is_empty() {
            return Ok(Some(256));
        }
        let bits = rest
            .parse::<usize>()
            .map_err(|_| crate::Error::InternalError(format!("invalid uint bits: {s}")))?;
        validate_int_bits(bits, "uint")?;
        return Ok(Some(bits));
    }
    Ok(None)
}

fn parse_int_bits(s: &str) -> crate::Result<Option<usize>> {
    if s == "int" {
        return Ok(Some(256));
    }
    if let Some(rest) = s.strip_prefix("int") {
        if rest.is_empty() {
            return Ok(Some(256));
        }
        let bits = rest
            .parse::<usize>()
            .map_err(|_| crate::Error::InternalError(format!("invalid int bits: {s}")))?;
        validate_int_bits(bits, "int")?;
        return Ok(Some(bits));
    }
    Ok(None)
}

fn validate_int_bits(bits: usize, label: &str) -> crate::Result<()> {
    if bits == 0 || bits > 256 || bits % 8 != 0 {
        return Err(crate::Error::InternalError(format!(
            "{label}{bits} must be 8..=256 and multiple of 8"
        )));
    }
    Ok(())
}

fn parse_bytes_n(s: &str) -> crate::Result<Option<usize>> {
    if let Some(rest) = s.strip_prefix("bytes") {
        if rest.is_empty() {
            return Ok(None);
        }
        let n = rest
            .parse::<usize>()
            .map_err(|_| crate::Error::InternalError(format!("invalid bytesN: {s}")))?;
        if n == 0 || n > 32 {
            return Err(crate::Error::InternalError(format!(
                "bytes{n} must be 1..=32"
            )));
        }
        return Ok(Some(n));
    }
    Ok(None)
}

fn parse_array_type(s: &str) -> crate::Result<Option<(&str, Option<usize>)>> {
    if let Some(i) = s.find('[') {
        let inner = &s[..i];
        let rest = &s[i..];
        if !rest.ends_with(']') {
            return Err(crate::Error::InternalError(format!(
                "malformed array type: {s}"
            )));
        }
        let inside = &rest[1..rest.len() - 1];
        if inside.is_empty() {
            return Ok(Some((inner, None)));
        }
        let n = inside.parse::<usize>().map_err(|_| {
            crate::Error::InternalError(format!("invalid fixed array size in: {s}"))
        })?;
        return Ok(Some((inner, Some(n))));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_sign_typed_data_arg() {
        let value = Value::from_str("{\"types\":{\"PermitDetails\":[{\"name\":\"token\",\"type\":\"address\"},{\"name\":\"amount\",\"type\":\"uint160\"},{\"name\":\"expiration\",\"type\":\"uint48\"},{\"name\":\"nonce\",\"type\":\"uint48\"}],\"PermitSingle\":[{\"name\":\"details\",\"type\":\"PermitDetails\"},{\"name\":\"spender\",\"type\":\"address\"},{\"name\":\"sigDeadline\",\"type\":\"uint256\"}],\"EIP712Domain\":[{\"name\":\"name\",\"type\":\"string\"},{\"name\":\"chainId\",\"type\":\"uint256\"},{\"name\":\"verifyingContract\",\"type\":\"address\"}]},\"domain\":{\"name\":\"Permit2\",\"chainId\":\"1\",\"verifyingContract\":\"0x000000000022d473030f116ddee9f6b43ac78ba3\"},\"primaryType\":\"PermitSingle\",\"message\":{\"details\":{\"token\":\"0xbc6736d346a5ebc0debc997397912cd9b8fae10a\",\"amount\":\"19930942425562201808\",\"expiration\":\"0\",\"nonce\":\"0\"},\"spender\":\"0xb53b4b2590457be63e1dcdaffa6a18ecd44d96d2\",\"sigDeadline\":\"115792089237316195423570985008687907853269984665640564039457584007913129639935\"}}").unwrap();
        let res = eip712_to_dyn(&value).unwrap();

        println!("res {:?}", res)
    }
}
