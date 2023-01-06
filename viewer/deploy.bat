@echo off
pushd "%~dp0"
for /F "tokens=*" %%g in ('git rev-parse --short HEAD') do (set git_commit=%%g)
git branch -D gh-pages
git worktree add --no-checkout --detach gh-pages
pushd gh-pages
git checkout --orphan gh-pages
git rm -rf .

xcopy /S "..\dist" .

git add -A
git commit -m "Deploy %git_commit%"
git push -f origin gh-pages
popd
git worktree remove --force gh-pages
popd
