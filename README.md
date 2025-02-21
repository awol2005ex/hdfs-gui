
<h1 align="center">
  Hdfs GUI
  <br>
</h1>

<h4 align="center">A Hdfs File GUI Software.</h4>


<p align="center">
  <a href="#key-features">Key Features</a> •
  <a href="#how-to-use">How To Use</a> •
  <a href="#download">Download</a> •
  <a href="#credits">Credits</a> •
  <a href="#related">Related</a> •
  <a href="#license">License</a>
</p>

![screenshot](doc\screenshot.png)

## Key Features

* Hdfs Files list, read and display , write , upload ,set permissions , delete files and directories , set file acls , show folder content summary.
* Hdfs Orc viewer ,show struct ,export to csv.
Support
* Cross platform
  - Windows, macOS and Linux ready.

## How To Use

To clone and run this application, you'll need [Git](https://git-scm.com) and [Node.js](https://nodejs.org/en/download/) (which comes with [npm](http://npmjs.com)) installed on your computer. From your command line:

```bash
# Clone this repository
$ git clone https://github.com/awol2005ex/hdfs-gui.git

# Go into the repository
$ cd hdfs-gui

# Install dependencies
$ npm install

# Run the app
$ npm run tauri build
```

# Hdfs namenode HA example

```
hdfs_url:hdfs://nameservice1

config: {
"dfs.nameservices":"nameservice1",
"dfs.namenode.rpc-address.nameservice1.namenode1":"node1:8020",
"dfs.namenode.rpc-address.nameservice1.namenode2":"node1:8020",
"dfs.ha.namenodes.nameservice1":"namenode1,namenode2",
"dfs.namenode.keytab.file":"/opt/xxx.keytab",
"dfs.namenode.keytab.enabled":"true",
"dfs.namenode.kerberos.principal":"xxx@XXX.COM"

}
```


## Download

You can [download](https://github.com/awol2005ex/hdfs-gui/releases/tag/v0.2.0) the latest installable version of hdfs-gui for Windows （install MT kerberos first and set PATH to the path of kinit）




## License

Apache License 2.0

---