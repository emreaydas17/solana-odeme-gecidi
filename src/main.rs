use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use std::env;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

// Sunucumuzun ana durumu (State). Veritabanı bağlantımızı burada tutacağız.
#[derive(Clone)]
struct AppState {
    db: PgPool,
}

#[tokio::main]
async fn main() {
    // 1. .env dosyasındaki gizli şifrelerimizi yüklüyoruz
    dotenv().ok();

    // 2. Veritabanı adresimizi alıyoruz
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL .env dosyasında bulunamadı!");

    println!("Veritabanına bağlanılıyor...");

    // 3. Neon.tech Postgres veritabanımıza bağlantı havuzu oluşturuyoruz
    let pool = PgPool::connect(&db_url)
        .await
        .expect("Veritabanına bağlanırken hata oluştu!");

    println!("Veritabanı bağlantısı başarılı!");

    // 4. schema.sql dosyamızı okuyup içindeki tabloyu (eğer yoksa) oluşturuyoruz
    let schema = include_str!("../schema.sql");
    sqlx::query(schema)
        .execute(&pool)
        .await
        .expect("Tablo oluşturulurken hata meydana geldi!");

    println!("Veritabanı tablosu hazır!");

    // Uygulama durumumuzu oluşturuyoruz
    let state = AppState { db: pool };

    // 5. Web API rotalarımızı tanımlıyoruz
    let app = Router::new()
        .route("/", get(ana_sayfa))
        .route("/siparis", post(siparis_olustur))
        .route("/dogrula", post(odeme_dogrula))
        .with_state(state)
        .layer(CorsLayer::permissive()); // YENİ EKLENEN SATIR (CORS İZNİ)

    // 6. Sunucumuzu 3000 portunda başlatıyoruz
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("Sunucu 3000 portunda çalışıyor: http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}

// Test için basit bir karşılama ucu
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
    // Benzersiz bir ID üretiyoruz
    let yeni_id = Uuid::new_v4().to_string();

    // Veritabanına kaydediyoruz
    // Ünlem işaretini kaldırdık ve .bind() kullandık
    let sonuc = sqlx::query("INSERT INTO siparisler (id, kullanici_email) VALUES ($1, $2)")
        .bind(&yeni_id)
        .bind(&payload.email)
        .execute(&state.db)
        .await;

    match sonuc {
        Ok(_) => Json(SiparisYaniti {
            mesaj: "Sipariş başarıyla oluşturuldu. Lütfen 1 USDC/USDT gönderip TxID'yi girin."
                .to_string(),
            siparis_id: yeni_id,
        }),
        Err(e) => {
            eprintln!("Veritabanı kayıt hatası: {:?}", e);
            Json(SiparisYaniti {
                mesaj: "Sipariş oluşturulurken sistemsel bir hata oluştu.".to_string(),
                siparis_id: "".to_string(),
            })
        }
    }
}

// ==========================================
// 2. ÖDEME DOĞRULAMA (SOLANA AĞI) BÖLÜMÜ
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
    // Solana Devnet RPC Adresi
    let rpc_url = "https://api.devnet.solana.com";
    let client = reqwest::Client::new();

    // Solana RPC'nin beklediği JSON formatı
    let request_body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getTransaction",
        "params": [
            payload.tx_id,
            {"encoding": "jsonParsed", "maxSupportedTransactionVersion": 0}
        ]
    });

    // Solana ağına isteği gönderiyoruz
    let res = client.post(rpc_url).json(&request_body).send().await;

    // Cevabı kontrol ediyoruz
    if let Ok(response) = res {
        let json_data: Value = response.json().await.unwrap_or_default();

        // Eğer işlem ağda varsa (result null değilse)
        if json_data["result"].is_object() {
            // Veritabanını 'odendi' olarak güncelle
            // Ünlem işaretini kaldırdık ve .bind() kullandık
            let guncelleme = sqlx::query(
                "UPDATE siparisler SET tx_id = $1, durum = 'odendi' WHERE id = $2 RETURNING id",
            )
            .bind(&payload.tx_id)
            .bind(&payload.siparis_id)
            .fetch_optional(&state.db)
            .await;

            match guncelleme {
                Ok(Some(_)) => {
                    return Json(DogrulamaYaniti {
                        mesaj: "Ödeme başarıyla doğrulandı! Ürün lisansınız: RUST-2026-XWZ"
                            .to_string(),
                        durum: "basarili".to_string(),
                    });
                }
                _ => {
                    return Json(DogrulamaYaniti {
                        mesaj: "Sipariş bulunamadı. Lütfen Sipariş ID'nizi kontrol edin."
                            .to_string(),
                        durum: "hata".to_string(),
                    });
                }
            }
        }
    }

    // İşlem bulunamazsa dönülecek hata
    Json(DogrulamaYaniti {
        mesaj: "Geçersiz TxID veya işlem henüz ağda onaylanmamış!".to_string(),
        durum: "hata".to_string(),
    })
}
