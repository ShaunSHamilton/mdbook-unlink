# Package target binary into a tarball
RELEASE_PATH=./target/x86_64-unknown-linux-gnu/release/mdbook-unlink
TARBALL_OUTPUT=./x86_64-unknown-linux-gnu.tar.gz

# Check if the binary exists
if [ ! -f $RELEASE_PATH ]; then
    echo "Binary not found at $RELEASE_PATH"
    exit 1
fi

# Create the tarball without the parent directory
tar -czvf $TARBALL_OUTPUT -C $(dirname $RELEASE_PATH) $(basename $RELEASE_PATH)
