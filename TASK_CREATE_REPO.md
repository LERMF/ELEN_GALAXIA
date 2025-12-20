# 📋 Task: Create Private GitHub Repository for NEXUS-PROTOCOL

> **Objective:** Publish local repository `/home/lermf/$PROJETOS$/kilo-sota-cloud` to a **private** GitHub repository.
> **Security Level:** High (Contains personal infrastructure configs)

---

## 🚀 Execution Steps

### 1. Preparation

- **Navigate to directory:**

  ```bash
  cd "/home/lermf/$PROJETOS$/kilo-sota-cloud"
  ```

- **Verify Git status:**

  ```bash
  git status
  # Ensure working tree is clean and 'feat: initial commit...' is present
  ```

### 2. Create Remote Repository (Choose One Method)

#### Option A: Using GitHub CLI (`gh`) - Recommended

If `gh` is installed and authenticated:

```bash
# Create private repo and push immediately
gh repo create nexus-protocol --private --source=. --remote=origin --push
```

#### Option B: Manual Creation (Browser)

1. Go to: [https://github.com/new](https://github.com/new)
2. **Repository name:** `nexus-protocol`
3. **Visibility:** Select **Private** 🔒 (Critical!)
4. **Initialize:** Do NOT check "Add a README", .gitignore, or license (we import existing code).
5. Click **Create repository**.

### 3. Link & Push

After creating the empty repo manually (Option B):

```bash
# Add remote
git remote add origin https://github.com/YOUR_USERNAME/nexus-protocol.git

# Set main branch (if not already)
git branch -M main

# Push all code
git push -u origin main
```

---

## 🛡️ Privacy Checklist

- [ ] Ensure repository visibility is **Private**.
- [ ] Verify `.gitignore` excludes sensitive files (secrets, personal keys).
- [ ] Do **not** publish `mcp_registry.json` if it contains unencrypted API keys (use template/env vars instead).

---

## 📝 Repository Description

**Suggested Description:**
> "NEXUS-PROTOCOL: Personal SOTA Cloud-Native Architecture & Global Configurations. Hardware-Zero Philosophy."
