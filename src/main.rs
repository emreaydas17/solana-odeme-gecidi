use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use dotenvy::dotenv;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use std::env;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    db: PgPool,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL .env dosyasında bulunamadı!");
    
    println!("Veritabanına bağlanılıyor...");
    let pool = PgPool::connect(&db_url).await.expect("Veritabanına bağlanırken hata oluştu!");
    println!("Veritabanı bağlantısı başarılı!");

    let schema = include_str!("../schema.sql");
    sqlx::query(schema).execute(&pool).await.expect("Tablo oluşturulurken hata meydana geldi!");
    println!("Veritabanı tablosu hazır!");

    let state = AppState { db: pool };

    let app = Router::new()
        .route("/", get(ana_sayfa))
        .route("/siparis", post(siparis_olustur))
        .route("/dogrula", post(odeme_dogrula))
        .with_state(state)
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Sunucu 3000 portunda çalışıyor...");
    
    axum::serve(listener, app).await.unwrap();
}

async fn ana_sayfa() -> &'static str {
    "Solana Ödeme Geçidi API'si Sorunsuz Çalışıyor!"
}

// ==========================================
// 1. SİPARİŞ OLUŞTURMA BÖLÜMÜ
// ==========================================

#[derive(Deserialize)]
struct SiparisIstegi {
    email: String,
}

#[derive(Serialize)]
struct SiparisYaniti {
    mesaj: String,
    siparis_id: String,
}

async fn siparis_olustur(
    State(state): State<AppState>,
    Json(payload): Json<SiparisIstegi>,
) -> Json<SiparisYaniti> {
    
    let email_regex = Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$").unwrap();
    if !email_regex.is_match(&payload.email) {
        return Json(SiparisYaniti {
            mesaj: "Lütfen geçerli bir e-posta adresi formatı girin!".to_string(),
            siparis_id: "".to_string(),
        });
    }

    let yeni_id = Uuid::new_v4().to_string();

    let sonuc = sqlx::query(
        "INSERT INTO siparisler (id, kullanici_email) VALUES ($1, $2)"
    )
    .bind(&yeni_id)
    .bind(&payload.email)
    .execute(&state.db)
    .await;

    match sonuc {
        Ok(_) => Json(SiparisYaniti {
            mesaj: "Sipariş oluşturuldu. Lütfen 1 USDC veya 1 USDT gönderip TxID'yi girin.".to_string(),
            siparis_id: yeni_id,
        }),
        Err(_) => Json(SiparisYaniti {
            mesaj: "Sipariş oluşturulurken sistemsel bir hata oluştu.".to_string(),
            siparis_id: "".to_string(),
        })
    }
}

// ==========================================
// 2. ÖDEME DOĞRULAMA (SOLANA AĞI - DERİN GÜVENLİK)
// ==========================================

#[derive(Deserialize)]
struct DogrulamaIstegi {
    siparis_id: String,
    tx_id: String,
}

#[derive(Serialize)]
struct DogrulamaYaniti {
    mesaj: String,
    durum: String,
}

async fn odeme_dogrula(
    State(state): State<AppState>,
    Json(payload): Json<DogrulamaIstegi>,
) -> Json<DogrulamaYaniti> {
    
    // --- 1. YENİ GÜVENLİK DUVARI: REPLAY ATTACK (ÇİFTE HARCAMA) KONTROLÜ ---
    let tx_kontrol = sqlx::query("SELECT id FROM siparisler WHERE tx_id = $1")
        .bind(&payload.tx_id)
        .fetch_optional(&state.db)
        .await;

    // Eğer veritabanında bu TxID zaten varsa, işlemi anında kesip atıyoruz!
    if let Ok(Some(_)) = tx_kontrol {
        return Json(DogrulamaYaniti {
            mesaj: "Güvenlik İhlali: Bu işlem kodu (TxID) daha önce kullanılmış!".to_string(),
            durum: "hata".to_string(),
        });
    }
    // ------------------------------------------------------------------------

    // DİKKAT: BURAYA KENDİ SOLANA DEVNET CÜZDAN ADRESİNİ YAZMALISIN!
    let magaza_cuzdani = "CY8YX95HbX1WZNe1YYNWjRuYkb1pCr1e6zcxgbFTeKho"; 
    
    let usdc_mint = "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU"; 
    let usdt_mint = "EJwZgeZrdC8TXTQbQBoL6bfuAnFUUy1PVCMB4DYPzVaS";

    let rpc_url = "https://api.devnet.solana.com";
    let client = reqwest::Client::new();
    
    let request_body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getTransaction",
        "params": [
            payload.tx_id,
            {"encoding": "jsonParsed", "maxSupportedTransactionVersion": 0}
        ]
    });

    let res = client.post(rpc_url).json(&request_body).send().await;

    if let Ok(response) = res {
        let json_data: Value = response.json().await.unwrap_or_default();
        
        if !json_data["result"].is_null() {
            let meta = &json_data["result"]["meta"];
            
            if !meta["err"].is_null() {
                return Json(DogrulamaYaniti {
                    mesaj: "Bu işlem Solana ağında başarısız olmuş (Failed).".to_string(),
                    durum: "hata".to_string(),
                });
            }

            let post_balances = meta["postTokenBalances"].as_array();
            let pre_balances = meta["preTokenBalances"].as_array();
            let mut odeme_gecerli = false;

            if let (Some(post), Some(pre)) = (post_balances, pre_balances) {
                for p in post {
                    let owner = p["owner"].as_str().unwrap_or("");
                    let mint = p["mint"].as_str().unwrap_or("");
                    let post_amount = p["uiTokenAmount"]["uiAmount"].as_f64().unwrap_or(0.0);
                    let account_index = p["accountIndex"].as_u64().unwrap_or(999);

                    if owner == magaza_cuzdani && (mint == usdc_mint || mint == usdt_mint) {
                        let mut pre_amount = 0.0;
                        for pr in pre {
                            if pr["accountIndex"].as_u64().unwrap_or(999) == account_index {
                                pre_amount = pr["uiTokenAmount"]["uiAmount"].as_f64().unwrap_or(0.0);
                                break;
                            }
                        }

                        let bakiye_farki = post_amount - pre_amount;

                        if bakiye_farki >= 0.99 {
                            odeme_gecerli = true;
                            break;
                        }
                    }
                }
            }

            if odeme_gecerli {
                let guncelleme = sqlx::query(
                    "UPDATE siparisler SET tx_id = $1, durum = 'odendi' WHERE id = $2 RETURNING id"
                )
                .bind(&payload.tx_id)
                .bind(&payload.siparis_id)
                .fetch_optional(&state.db)
                .await;

                match guncelleme {
                    Ok(Some(_)) => return Json(DogrulamaYaniti {
                        mesaj: "Ödeme başarıyla doğrulandı! Lisans: RUST-2026-XWZ".to_string(),
                        durum: "basarili".to_string(),
                    }),
                    _ => return Json(DogrulamaYaniti {
                        mesaj: "Sipariş bulunamadı veya sistem hatası.".to_string(),
                        durum: "hata".to_string(),
                    }),
                }
            } else {
                return Json(DogrulamaYaniti {
                    mesaj: "İşlem bulundu ancak: Alıcı yanlış, Token geçersiz veya Tutar 1'den az!".to_string(),
                    durum: "hata".to_string(),
                });
            }
        }
    }

    Json(DogrulamaYaniti {
        mesaj: "Geçersiz TxID veya işlem ağda yok!".to_string(),
        durum: "hata".to_string(),
    })
}