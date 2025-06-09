git remote set-url origin https://$GIT_ACCESS_TOKEN@github.com/$GIT_REPOSITORY
git reset --hard
git fetch
git checkout $GIT_BRANCH
git pull

# Install dependencies
bun i --production

# Start the application
export FORCE_COLOR=1
export NODE_ENV=production
bun run start 2>&1 | tee -a latest.log