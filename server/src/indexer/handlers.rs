use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};
use sqlx::{PgPool, Row};
use stellar_xdr::next::{ScVal, Limits, ReadXdr};
use tracing::info;
use uuid::Uuid;

use crate::websocket::WsState;
use crate::escrow::EscrowEvent as WsEscrowEvent;
use super::types::SorobanEvent;

pub struct EventHandler {
    pool: PgPool,
    ws_state: Option<WsState>,
}

impl EventHandler {
    pub fn new(pool: PgPool, ws_state: Option<WsState>) -> Self {
        Self { pool, ws_state }
    }

    pub async fn handle_event(&self, event: &SorobanEvent, contract_type: &str) -> Result<()> {
        let decoded_topics = decode_topics(&event.topic)?;
        if decoded_topics.is_empty() {
            return Ok(());
        }

        let event_name = match &decoded_topics[0] {
            ScVal::Symbol(s) => s.to_string(),
            _ => return Ok(()),
        };

        let value_xdr = general_purpose::STANDARD.decode(&event.value.xdr)?;
        let data = ScVal::from_xdr(&value_xdr, Limits::none())?;

        match contract_type {
            "collateral" => self.handle_collateral_event(&event_name, &data).await?,
            "escrow" => self.handle_escrow_event(&event_name, &data).await?,
            "loan" => self.handle_loan_event(&event_name, &data).await?,
            _ => info!("Unknown contract type: {}", contract_type),
        }

        Ok(())
    }

    async fn handle_collateral_event(&self, name: &str, data: &ScVal) -> Result<()> {
        match name {
            "coll_reg" => {
                if let ScVal::Vec(Some(args)) = data {
                    if args.len() < 4 { return Err(anyhow!("Invalid args length")); }
                    let id = scval_to_u64(&args[0])?;
                    let owner = scval_to_address(&args[1])?;
                    let face_value = scval_to_i128(&args[2])?;
                    let expiry_ts = scval_to_u64(&args[3])?;
                    let metadata_placeholder = format!("hash_{}", id); 

                    sqlx::query(
                        r#"
                        INSERT INTO collateral (collateral_id, owner_id, face_value, expiry_ts, metadata_hash, registered_at, status)
                        VALUES ($1, (SELECT id FROM users WHERE stellar_address = $2 LIMIT 1), $3, $4, $5, NOW(), 'active')
                        ON CONFLICT (collateral_id) DO NOTHING
                        "#
                    )
                    .bind(id as i64)
                    .bind(owner)
                    .bind(face_value as i64)
                    .bind(expiry_ts as i64)
                    .bind(metadata_placeholder)
                    .execute(&self.pool)
                    .await?;
                }
            },
            "coll_lock" => {
                if let ScVal::Vec(Some(args)) = data {
                    let id = scval_to_u64(&args[0])?;
                    sqlx::query(
                        "UPDATE collateral SET locked = true, status = 'locked' WHERE collateral_id = $1"
                    )
                    .bind(id as i64)
                    .execute(&self.pool)
                    .await?;
                }
            },
            "coll_unlk" => {
                 if let ScVal::Vec(Some(args)) = data {
                    let id = scval_to_u64(&args[0])?;
                    sqlx::query(
                        "UPDATE collateral SET locked = false, status = 'active' WHERE collateral_id = $1"
                    )
                    .bind(id as i64)
                    .execute(&self.pool)
                    .await?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_escrow_event(&self, name: &str, data: &ScVal) -> Result<()> {
        match name {
            "esc_crtd" => {
                if let ScVal::Vec(Some(args)) = data {
                    let id = scval_to_u64(&args[0])?;
                    let buyer_addr = scval_to_address(&args[1])?;
                    let seller_addr = scval_to_address(&args[2])?;
                    let amount = scval_to_i128(&args[3])?;

                    // We need to fetch UUIDs for WS event
                    let buyer_id = self.get_user_id_by_address(&buyer_addr).await?;
                    let seller_id = self.get_user_id_by_address(&seller_addr).await?;

                    if let (Some(bid), Some(sid)) = (buyer_id, seller_id) {
                        sqlx::query(
                            r#"
                            INSERT INTO escrows (escrow_id, buyer_id, seller_id, amount, status, collateral_id, oracle_address, release_conditions)
                            VALUES ($1, $2, $3, $4, 'pending', 'unknown', 'unknown', 'unknown')
                            ON CONFLICT (escrow_id) DO NOTHING
                            "#
                        )
                        .bind(id as i64)
                        .bind(bid)
                        .bind(sid)
                        .bind(amount as i64)
                        .execute(&self.pool)
                        .await?;

                        if let Some(ws) = &self.ws_state {
                            ws.broadcast_event(WsEscrowEvent::Created { 
                                escrow_id: id as i64,
                                buyer_id: bid,
                                seller_id: sid
                            }).await;
                        }
                    } else {
                        info!("Buyer or Seller not found for escrow {}", id);
                    }
                }
            },
             "esc_act" => {
                if let ScVal::Vec(Some(args)) = data {
                    let id = scval_to_u64(&args[0])?;
                    sqlx::query(
                        "UPDATE escrows SET status = 'active'::escrow_status WHERE escrow_id = $1"
                    )
                    .bind(id as i64)
                    .execute(&self.pool)
                    .await?;

                    if let Some(ws) = &self.ws_state {
                        ws.broadcast_event(WsEscrowEvent::Activated { escrow_id: id as i64 }).await;
                    }
                }
            },
             "esc_rel" => {
                if let ScVal::Vec(Some(args)) = data {
                    let id = scval_to_u64(&args[0])?;
                    sqlx::query(
                        "UPDATE escrows SET status = 'released'::escrow_status WHERE escrow_id = $1"
                    )
                    .bind(id as i64)
                    .execute(&self.pool)
                    .await?;
                    
                    if let Some(ws) = &self.ws_state {
                        ws.broadcast_event(WsEscrowEvent::Released { escrow_id: id as i64 }).await;
                    }
                }
            },
            _ => {}
        }
        Ok(())
    }

    async fn handle_loan_event(&self, name: &str, _data: &ScVal) -> Result<()> {
        match name {
            "loan_iss" => {
                // ... (implementation pending schema confirmation)
            },
            _ => {}
        }
        Ok(())
    }

    async fn get_user_id_by_address(&self, address: &str) -> Result<Option<Uuid>> {
        let rec = sqlx::query("SELECT id FROM users WHERE stellar_address = $1")
            .bind(address)
            .fetch_optional(&self.pool)
            .await?;
        
        Ok(rec.map(|r| r.get("id")))
    }
}

// Helpers
fn decode_topics(topics: &[String]) -> Result<Vec<ScVal>> {
    let mut res = Vec::new();
    for t in topics {
        let bytes = general_purpose::STANDARD.decode(t)?;
        let val = ScVal::from_xdr(&bytes, Limits::none())?;
        res.push(val);
    }
    Ok(res)
}

fn scval_to_u64(val: &ScVal) -> Result<u64> {
     match val {
        ScVal::U64(v) => Ok(*v),
        ScVal::I64(v) => Ok(*v as u64),
        ScVal::U32(v) => Ok(*v as u64),
        ScVal::I32(v) => Ok(*v as u64),
        _ => Err(anyhow!("Expected U64-like")),
    }
}

fn scval_to_i128(val: &ScVal) -> Result<i128> {
    match val {
        ScVal::I128(v) => Ok(i128::from(v.lo) | ((i128::from(v.hi)) << 64)),
        ScVal::U64(v) => Ok(*v as i128),
        ScVal::I64(v) => Ok(*v as i128),
        _ => Err(anyhow!("Expected I128-like")),
    }
}

fn scval_to_address(val: &ScVal) -> Result<String> {
    match val {
        ScVal::Address(addr) => Ok(addr.to_string()),
        _ => Err(anyhow!("Expected Address")),
    }
}
