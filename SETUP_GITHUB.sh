#!/bin/bash
# SETUP_GITHUB.sh
# Helper script to push ELEN_GALAXIA to GitHub

echo "🚀 ELEN_GALAXIA GitHub Setup"
echo "---------------------------"
echo "⚠️  IMPORTANT: Please ensure you create a **PRIVATE** repository."
echo "Since 'gh' CLI is not available, please manually create a PRIVATE repository named 'ELEN_GALAXIA' on GitHub."
echo ""
echo "🔗 URL should look like: https://github.com/<YOUR_USERNAME>/ELEN_GALAXIA.git"
echo ""
read -p "📝 Paste your new repository URL here: " REPO_URL

if [ -z "$REPO_URL" ]; then
    echo "❌ No URL provided. Exiting."
    exit 1
fi

echo "🔄 Adding remote origin..."
git remote add origin "$REPO_URL"

echo "⬆️ Pushing code to main branch..."
git push -u origin main

echo "✅ Done! ELEN_GALAXIA is live and PRIVATE."
