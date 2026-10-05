#!/usr/bin/env bash
# Kullanım: release.sh NN kisa-ad 0.X.0 "commit mesajı"
set -euo pipefail
NN="$1"
SLUG="$2"
V="$3"
MSG="$4"
BRANCH="faz/$NN-$SLUG"
TAG="v$V"
TRAILER="Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"

if [ "$(git branch --show-current)" != "$BRANCH" ]; then
  git switch "$BRANCH" 2> /dev/null || git switch -c "$BRANCH"
fi
git add -A
node scripts/secret-scan.mjs --staged
git diff --cached --quiet || git commit -m "$MSG" -m "$TRAILER"
git push -u origin "$BRANCH"

git switch main
git pull --ff-only origin main || true
git merge --no-ff "$BRANCH" -m "Merge $BRANCH" -m "$TRAILER"
git tag -a "$TAG" -m "Oyun Arşivim $TAG"
git push origin main "$TAG"

sleep 10
for wf in ci.yml release.yml; do
  id=$(gh run list --workflow "$wf" --branch "$( [ "$wf" = release.yml ] && echo "$TAG" || echo main)" \
    --limit 1 --json databaseId -q '.[0].databaseId')
  echo "İzleniyor: $wf ($id)"
  gh run watch "$id" --exit-status
done
gh release view "$TAG" --json url -q .url
