# ⚡ Cloudflare Pages - 5 Dakikada Deploy

REVM dokümantasyonunu Cloudflare Pages'e deploy etmek için **sadece 3 adım**:

## 1️⃣ Cloudflare API Bilgilerini Alın (2 dakika)

### API Token:
1. https://dash.cloudflare.com/ → **Profile** → **API Tokens**
2. **Create Token** → **Edit Cloudflare Workers** template kullanın
3. Token'ı kopyalayın (**bir daha göremezsiniz!**)

### Account ID:
1. Cloudflare Dashboard'da herhangi bir siteye tıklayın
2. Sağ tarafta **Account ID** görünür, kopyalayın

## 2️⃣ GitHub Secrets Ekleyin (1 dakika)

GitHub Repository → **Settings** → **Secrets and variables** → **Actions** → **New repository secret**

**İki secret ekleyin:**

| Secret Name | Value |
|------------|--------|
| `CLOUDFLARE_API_TOKEN` | (Cloudflare'den aldığınız token) |
| `CLOUDFLARE_ACCOUNT_ID` | (Cloudflare Account ID) |

## 3️⃣ Workflow'u Push Edin (2 dakika)

```bash
# Workflow dosyasını commit edin
git add .github/workflows/cloudflare-pages.yml CLOUDFLARE_SETUP.md
git commit -m "Add Cloudflare Pages deployment

Co-Authored-By: Warp <agent@warp.dev>"
git push origin main
```

**İşte bu kadar! 🎉**

---

## ✅ Deploy Sonuçlarını Kontrol Edin

1. **GitHub Actions**: https://github.com/bluealloy/revm/actions
2. **Cloudflare Dashboard**: https://dash.cloudflare.com/ → **Workers & Pages**
3. **Live Site**: https://revm-docs.pages.dev

---

## 🚀 Site Yapısı

```
https://revm-docs.pages.dev/
├── /                          → Ana sayfa (REVM docs portal)
├── /book/                     → mdBook dokümantasyonu
└── /docs/revm/                → Rust API dokümantasyonu
```

---

## 🔧 Sorun mu Yaşıyorsunuz?

### "Deployment failed" hatası:
- GitHub Secrets'ı kontrol edin (Settings → Secrets)
- Cloudflare API Token'ın **Cloudflare Pages Edit** yetkisi olmalı

### "Project not found" hatası:
Cloudflare Dashboard'da manuel proje oluşturun:
- **Workers & Pages** → **Create application** → **Pages**
- Proje adı: `revm-docs`

### Build çok uzun sürüyor:
Normal! İlk build ~5-10 dakika sürebilir. Sonraki build'ler cache sayesinde ~2-3 dakika.

---

## 📚 Detaylı Dokümantasyon

Tüm detaylar için: **[CLOUDFLARE_SETUP.md](./CLOUDFLARE_SETUP.md)**

- Custom domain ekleme
- Build optimizasyonları
- Analytics kurulumu
- Environment protection
- Ve daha fazlası...

---

## 💡 Pro İpuçları

### PR Preview URL'leri
Her Pull Request otomatik olarak preview URL alır:
- `https://BRANCH_NAME.revm-docs.pages.dev`

### Manuel Deploy
GitHub Actions UI'dan manuel deploy:
1. **Actions** → **Deploy to Cloudflare Pages**
2. **Run workflow** → Branch seçin → **Run**

### Deployment Bildirimleri
Cloudflare Dashboard → **Workers & Pages** → **revm-docs** → **Settings** → **Notifications**

---

**Başarılar! 🚀**
