# Cloudflare Pages Deployment Setup

Bu doküman, REVM projesini Cloudflare Pages'e otomatik deploy etmek için gerekli adımları içerir.

## 🚀 Hızlı Başlangıç

### 1. Cloudflare API Token ve Account ID Alma

#### A. Cloudflare Dashboard'a girin:
https://dash.cloudflare.com/

#### B. API Token oluşturun:
1. **Profile** > **API Tokens** > **Create Token**
2. **Use Template**: "Edit Cloudflare Workers"
3. Veya **Custom Token** ile şu izinleri verin:
   - **Account** > **Cloudflare Pages** > **Edit**
4. **Continue to Summary** > **Create Token**
5. Token'ı kopyalayıp güvenli bir yere kaydedin (bir daha göremezsiniz!)

#### C. Account ID'yi bulun:
1. Cloudflare Dashboard'da herhangi bir siteye tıklayın
2. Sağ taraftaki **Overview** sekmesinde **Account ID** görünür
3. Kopyalayın

**Alternatif yol:**
```bash
# Cloudflare API ile Account ID öğrenme
curl -X GET "https://api.cloudflare.com/client/v4/accounts" \
  -H "Authorization: Bearer YOUR_API_TOKEN" \
  -H "Content-Type: application/json"
```

### 2. GitHub Repository Secrets Ekleme

#### GitHub Repository'de:
1. **Settings** > **Secrets and variables** > **Actions**
2. **New repository secret** butonuna tıklayın
3. İki secret ekleyin:

**Secret 1:**
- **Name**: `CLOUDFLARE_API_TOKEN`
- **Value**: (Yukarıda oluşturduğunuz token)

**Secret 2:**
- **Name**: `CLOUDFLARE_ACCOUNT_ID`
- **Value**: (Cloudflare Account ID)

### 3. Cloudflare Pages Projesi Oluşturma (Opsiyonel)

Cloudflare otomatik olarak projeyi oluşturabilir, ama manuel oluşturmak isterseniz:

1. **Cloudflare Dashboard** > **Workers & Pages** > **Create application** > **Pages**
2. **Connect to Git** VEYA **Direct Upload**
3. Proje adı: `revm-docs` (workflow dosyasındaki `projectName` ile aynı olmalı)
4. **Framework preset**: None
5. **Build command**: (boş bırakın, GitHub Actions hallediyor)
6. **Build output directory**: `cloudflare-output`

### 4. Workflow'u Test Etme

#### A. Değişiklikleri commit edin:
```bash
git add .github/workflows/cloudflare-pages.yml
git commit -m "Add Cloudflare Pages deployment workflow

Co-Authored-By: Warp <agent@warp.dev>"
git push origin main
```

#### B. GitHub Actions sekmesinde deploy'u izleyin:
https://github.com/bluealloy/revm/actions

#### C. Deploy başarılıysa, site şu adreste olacak:
- **Production**: https://revm-docs.pages.dev
- **Preview (PR)**: https://BRANCH_NAME.revm-docs.pages.dev

---

## 🔧 Yapılandırma Seçenekleri

### Proje Adını Değiştirme

`.github/workflows/cloudflare-pages.yml` dosyasında:

```yaml
projectName: revm-docs  # İstediğiniz adı verin
```

### Custom Domain Ekleme

Cloudflare Pages Dashboard'da:
1. **Workers & Pages** > **revm-docs** > **Custom domains**
2. **Set up a custom domain**
3. Domain adınızı girin (örn: `docs.revm.io`)
4. DNS kayıtlarını ekleyin

### Build Süresini Kısaltma

Eğer her push'ta hem mdBook hem API docs build etmek istemiyorsanız:

```yaml
# Sadece mdBook build et
- name: Build mdBook Documentation
  run: |
    cp README.md book/src/README.md
    sed -i -e 's|../../README.md|./README.md|g' book/src/SUMMARY.md
    mdbook build book

# API docs'ı atla (veya sadece main branch'te build et)
- name: Build Rust API Documentation
  if: github.ref == 'refs/heads/main'  # Sadece main'de
  run: |
    RUSTDOCFLAGS="--enable-index-page -Zunstable-options" cargo doc --all --no-deps
```

### Preview Branch'leri Filtreleme

Sadece belirli branch'lerde deploy yapmak için:

```yaml
on:
  push:
    branches: 
      - main
      - dev
      - staging
  pull_request:
    branches: [main]
```

---

## 📊 Site Yapısı

Deploy edilen site şu yapıda olacak:

```
https://revm-docs.pages.dev/
├── index.html              # Ana giriş sayfası (otomatik oluşturulur)
├── book/                   # mdBook dokümantasyonu
│   ├── index.html
│   ├── awesome.html
│   └── ...
└── docs/                   # Rust API dokümantasyonu
    ├── revm/
    │   └── index.html
    └── ...
```

**URL'ler:**
- Ana sayfa: `https://revm-docs.pages.dev/`
- mdBook: `https://revm-docs.pages.dev/book/index.html`
- API Docs: `https://revm-docs.pages.dev/docs/revm/index.html`

---

## 🛠️ Sorun Giderme

### Deployment başarısız oluyor

#### 1. Secrets kontrol edin:
```bash
# GitHub CLI ile kontrol
gh secret list
```

#### 2. Cloudflare API Token izinlerini doğrulayın:
- **Account** > **Cloudflare Pages** > **Edit** yetkisi olmalı

#### 3. Logs'u inceleyin:
- GitHub Actions > Workflow run > Job logs

### "Project not found" hatası

Cloudflare Pages Dashboard'da manuel olarak proje oluşturun:
- Proje adı workflow dosyasındaki `projectName` ile aynı olmalı

### Build çok uzun sürüyor

#### Cache kullanımı artırın:
```yaml
- uses: Swatinem/rust-cache@v2
  with:
    cache-on-failure: true
    shared-key: "revm-docs"  # Cache key
```

#### Sadece değişen dosyaları build edin:
```yaml
# Git diff ile sadece docs değiştiyse cargo doc çalıştır
- name: Check if docs changed
  id: docs-changed
  run: |
    if git diff --name-only ${{ github.event.before }} ${{ github.sha }} | grep -qE '^(src/|crates/)'; then
      echo "changed=true" >> $GITHUB_OUTPUT
    fi

- name: Build Rust API Documentation
  if: steps.docs-changed.outputs.changed == 'true'
  run: cargo doc --all --no-deps
```

---

## 🔒 Güvenlik

### Environment Protection

GitHub'da Production environment korumasi ekleyin:

1. **Settings** > **Environments** > **New environment**
2. **Environment name**: `cloudflare-production`
3. **Required reviewers** ekleyin (opsiyonel)
4. **Deployment branches**: Sadece `main`

Sonra workflow'da:

```yaml
deploy:
  environment:
    name: cloudflare-production
    url: https://revm-docs.pages.dev
```

### Secrets Rotation

API token'ları düzenli olarak yenileyin:
1. Cloudflare'de yeni token oluşturun
2. GitHub Secrets'ı güncelleyin
3. Eski token'ı devre dışı bırakın

---

## 📈 Analytics ve Monitoring

### Cloudflare Web Analytics

Cloudflare Dashboard > Pages > revm-docs > **Web Analytics**
- Visitor stats
- Performance metrics
- Geographic distribution

### GitHub Actions Monitoring

```yaml
- name: Deployment Status
  if: always()
  run: |
    echo "Deployment completed!"
    echo "URL: https://revm-docs.pages.dev"
```

---

## 🎯 Gelişmiş Özellikler

### A. Otomatik Lighthouse CI

`.github/workflows/cloudflare-pages.yml` içine ekleyin:

```yaml
- name: Run Lighthouse CI
  uses: treosh/lighthouse-ci-action@v10
  with:
    urls: |
      https://revm-docs.pages.dev
      https://revm-docs.pages.dev/book/index.html
    uploadArtifacts: true
```

### B. Slack/Discord Bildirimleri

```yaml
- name: Notify on Success
  if: success()
  run: |
    curl -X POST ${{ secrets.SLACK_WEBHOOK_URL }} \
      -H 'Content-Type: application/json' \
      -d '{"text":"✅ REVM docs deployed to Cloudflare Pages!"}'
```

### C. Preview URL Comment on PR

Workflow zaten PR'lara otomatik yorum ekliyor! PR oluşturduğunuzda deployment URL'sini göreceksiniz.

---

## 📚 Kaynaklar

- [Cloudflare Pages Docs](https://developers.cloudflare.com/pages/)
- [GitHub Actions Cloudflare Plugin](https://github.com/cloudflare/pages-action)
- [mdBook Guide](https://rust-lang.github.io/mdBook/)
- [Rustdoc Book](https://doc.rust-lang.org/rustdoc/)

---

## ✅ Checklist

- [ ] Cloudflare API Token oluşturuldu
- [ ] Cloudflare Account ID bulundu
- [ ] GitHub Secrets eklendi (`CLOUDFLARE_API_TOKEN`, `CLOUDFLARE_ACCOUNT_ID`)
- [ ] Workflow dosyası commit edildi
- [ ] İlk deployment başarılı
- [ ] Site erişilebilir: https://revm-docs.pages.dev
- [ ] Custom domain eklendi (opsiyonel)
- [ ] Analytics aktif edildi (opsiyonel)

---

**İyi dokümantasyonlar! 🚀**
