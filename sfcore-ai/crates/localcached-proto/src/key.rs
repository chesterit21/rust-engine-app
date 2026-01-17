use crate::ProtoError;

pub fn validate_key_3parts(key: &str) -> Result<(&str, &str, &str), ProtoError> {
    let mut it = key.splitn(3, ':');
    let svc = it.next().ok_or(ProtoError::InvalidKeyFormat)?;
    let table = it.next().ok_or(ProtoError::InvalidKeyFormat)?;
    let pk = it.next().ok_or(ProtoError::InvalidKeyFormat)?;
    if svc.is_empty() || table.is_empty() || pk.is_empty() {
        return Err(ProtoError::InvalidKeyFormat);
    }
    Ok((svc, table, pk))
}

pub fn topic_from_key(key: &str) -> Result<String, ProtoError> {
    let (svc, table, _) = validate_key_3parts(key)?;
    Ok(format!("t:{svc}:{table}"))
}

pub fn validate_topic(topic: &str) -> Result<(&str, &str), ProtoError> {
    // expected "t:svc:table"
    let mut it = topic.splitn(3, ':');
    let prefix = it.next().unwrap_or("");
    let svc = it.next().ok_or(ProtoError::BadPayload)?;
    let table = it.next().ok_or(ProtoError::BadPayload)?;
    if prefix != "t" || svc.is_empty() || table.is_empty() {
        return Err(ProtoError::BadPayload);
    }
    Ok((svc, table))
}
