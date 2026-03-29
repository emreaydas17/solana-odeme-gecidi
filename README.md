# 🚀 Solana Web3 Payment Gateway | Kripto Ödeme Geçidi

[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](#)
[![Solana](https://img.shields.io/badge/Solana-14F195?style=for-the-badge&logo=solana&logoColor=white)](#)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-316192?style=for-the-badge&logo=postgresql&logoColor=white)](#)
[![TailwindCSS](https://img.shields.io/badge/Tailwind_CSS-38B2AC?style=for-the-badge&logo=tailwind-css&logoColor=white)](#)

*Read this document in **[English](#english)** or **[Türkçe](#türkçe)**.*

---

<a id="english"></a>
## 🇬🇧 English

### 📌 About the Project
This project is a Full-Stack Web3 payment gateway designed for modern e-commerce systems. It allows merchants to accept digital payments (USDC/USDT) directly via the Solana blockchain without paying third-party commission fees. It is built with a decoupled architecture, ensuring high performance, state-of-the-art security, and a seamless user experience.

### 🛡️ Key Features & Security (Deep Validation)
Unlike standard payment gateways that only check if a transaction exists, this system implements deep blockchain analysis:
- **Balance Delta & Mint Validation:** It parses the transaction via Solana RPC to ensure the exact token (USDC or USDT contract addresses) was sent, and validates the exact amount received by checking pre/post account balances.
- **Replay Attack Protection:** Cross-references incoming Transaction Hashes (TxID) with the PostgreSQL database to instantly block double-spending attempts.
- **Data Integrity:** Strict regex-based email validation on both the frontend and the Rust backend.
- **Instant Reveal:** Once the blockchain confirms the exact payment, the digital product (license code) is instantly revealed on the UI, reducing e-commerce friction to zero.

### 💻 Tech Stack & Architecture
* **Backend API:** Rust, Axum, SQLx, Tokio (Deployed on Render.com)
* **Database:** PostgreSQL (Serverless via Neon.tech)
* **Blockchain:** Solana Devnet JSON RPC API
* **Frontend:** HTML5, Vanilla JavaScript, Tailwind CSS (Deployed on Netlify)

### ⚙️ How It Works
1. User enters their email on the UI.
2. Rust API generates a unique Order ID and logs it into PostgreSQL.
3. User transfers at least 1 USDC/USDT to the merchant's Solana wallet and submits the TxID.
4. The Backend securely queries the Solana RPC, validates the token mint, receiver address, and exact delta amount.
5. If valid and unused (Replay Attack check), the DB is updated and the UI displays the product.

---

<a id="türkçe"></a>
## 🇹🇷 Türkçe

### 📌 Proje Hakkında
Bu proje, modern e-ticaret sistemleri için tasarlanmış, Rust ve Solana altyapısı kullanan Full-Stack bir Web3 ödeme geçididir. Satıcıların 3. parti komisyonları ödemeden, doğrudan blockchain üzerinden dijital ödemeler (USDC/USDT) alabilmesini sağlar. Yüksek performans, üst düzey güvenlik ve kusursuz bir kullanıcı deneyimi (UX) için dağıtık bulut mimarisiyle (Decoupled Architecture) inşa edilmiştir.

### 🛡️ Öne Çıkan Güvenlik Özellikleri (Deep Validation)
Sıradan ödeme sistemlerinin aksine, bu altyapı blockchain üzerindeki verileri derinlemesine analiz eder:
- **Tutar ve Token (Mint) Doğrulaması:** Solana RPC üzerinden işlem verisini parçalayarak (Parse), gelen varlığın sahte tokenlar değil, orijinal USDC/USDT kontratları olduğunu ve tutarın eksiksiz ulaştığını bakiye farkı (Delta) ile teyit eder.
- **Replay Attack (Çifte Harcama) Koruması:** Sisteme girilen İşlem Hash'lerini (TxID) anlık olarak PostgreSQL veritabanındaki kayıtlarla eşleştirir. Daha önce kullanılmış bir ödeme kodu tekrar girilirse sistem işlemi anında reddeder.
- **Veri Doğrulama:** Ön yüz ve arka uçta (Rust) Regex tabanlı katı e-posta doğrulaması yapılarak API suistimalleri önlenir.
- **Anında Teslimat (Instant Reveal):** Blockchain onayı alındığı milisaniye içerisinde dijital ürün (lisans kodu) ekranda belirir. E-ticaret akışındaki bekleme süresi (Friction) sıfıra indirilmiştir.

### 💻 Kullanılan Teknolojiler ve Mimari
* **Arka Uç (Backend):** Rust, Axum, SQLx, Tokio (Render.com üzerinde canlıda)
* **Veritabanı:** PostgreSQL (Neon.tech Serverless)
* **Blockchain:** Solana Devnet JSON RPC API
* **Ön Yüz (Frontend):** HTML5, Vanilla JavaScript, Tailwind CSS (Netlify üzerinde canlıda)

### ⚙️ Nasıl Çalışır?
1. Kullanıcı şık önyüz üzerinden e-posta adresini girerek süreci başlatır.
2. Rust API benzersiz bir Sipariş ID'si oluşturup durumu veritabanına yazar.
3. Kullanıcı belirtilen cüzdana en az 1 USDC/USDT gönderip TxID'yi sisteme girer.
4. Sunucu, Solana RPC'ye bağlanır; alıcı cüzdanı, token türünü, bakiye farkını ve DB geçmişini (Replay Attack) kontrol eder.
5. Tüm güvenlik duvarları aşılırsa ürün (lisans kodu) anında ekranda teslim edilir.

<p align="center">
  <video src="https://github.com/user-attachments/assets/0a59c641-bb67-4b7b-870f-1156ae379013" controls="controls" style="max-width: 70%; height: auto;">
    Tarayıcınız video etiketini desteklemiyor.
  </video>
</p>

<br>
