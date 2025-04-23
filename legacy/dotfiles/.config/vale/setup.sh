# Make a temporary directory
mkdir temp
cd temp

# Download the gitlab vale style
wget https://gitlab.com/gitlab-org/gitlab/-/archive/master/gitlab-master.tar.gz?path=doc/.vale/gitlab
mv gitlab-master.tar.gz\?path=doc%2F.vale%2Fgitlab gitlab-master.tar.gz

# Extract the vale style
tar -xvf gitlab-master.tar.gz

# Move it into the styles dir
mv gitlab-master-doc-.vale-gitlab/doc/.vale/gitlab ../styles/

# Remove the temp directory
cd ..
rm -rf temp
