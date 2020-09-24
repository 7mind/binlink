use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct ResolvedConfig {
    pub bins: HashMap<String, String>,
    pub names: HashSet<String>,
}

#[derive(Debug)]
pub struct Config {
    pub local: Option<LocalConfig>,
    pub global: Option<GlobalConfig>,
}


impl Config {
    pub fn resolve(self) -> ResolvedConfig {
        let globals: (HashMap<String, String>, HashSet<String>) = match self.global {
            None => {
                (HashMap::new(), HashSet::new())
            }
            Some(e) => {
                (e.resolve(), e.bins.into_iter().map(|b| b.name).collect())
            }
        };

        let locals: (HashMap<String, String>, HashSet<String>) = match self.local {
            None => {
                (HashMap::new(), HashSet::new())
            }
            Some(e) => {
                (e.resolve(), e.bins.into_iter().map(|b| b.name).collect())
            }
        };
        let out = globals.0.into_iter().chain(locals.0).collect();
        let outN = globals.1.into_iter().chain(locals.1).collect();


        ResolvedConfig { bins: out, names: outN }
    }
}

trait AbstractConfig {
    fn bins(&self) -> &Vec<LinkedBinary>;
    fn kits(&self) -> &Vec<KitConfig>;
    fn paths(&self) -> &Vec<KitPath>;

    fn resolve(&self) -> HashMap<String, String> {
        let paths: HashMap<String, String> = self.paths().iter().map(|p| {
            let dir = match &p.target {
                SdkTarget::Dir { path } => {
                    String::from(path)
                }
            };

            (String::from(&p.id), dir)
        }).collect();

        let kits: HashMap<String, String> = self.kits().iter().flat_map(|kit| {
            match paths.get(kit.id.as_str()) {
                Some(path) => {
                    [(String::from(&kit.name), String::from(path))].to_vec()
                }
                None => {
                    Vec::new()
                }
            }
        }).collect();


        let bins: HashMap<String, String> = self.bins().iter().flat_map(|bin| {
            match &bin.target {
                LinkTarget::Sdk { name } => {
                    match kits.get(name.as_str()) {
                        Some(path) => {
                            [(String::from(&bin.name), String::from(path))].to_vec()
                        }
                        None => {
                            Vec::new()
                        }
                    }
                }
                LinkTarget::Default => {
                    Vec::new()
                }
            }
        }).collect();

        bins
    }
}

impl AbstractConfig for LocalConfig {
    fn bins(&self) -> &Vec<LinkedBinary> {
        &self.bins
    }

    fn kits(&self) -> &Vec<KitConfig> {
        &self.kits
    }

    fn paths(&self) -> &Vec<KitPath> {
        &self.paths
    }
}

impl AbstractConfig for GlobalConfig {
    fn bins(&self) -> &Vec<LinkedBinary> {
        &self.bins
    }

    fn kits(&self) -> &Vec<KitConfig> {
        &self.kits
    }

    fn paths(&self) -> &Vec<KitPath> {
        &self.paths
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LocalConfig {
    pub bins: Vec<LinkedBinary>,
    pub kits: Vec<KitConfig>,
    pub paths: Vec<KitPath>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GlobalConfig {
    pub bins: Vec<LinkedBinary>,
    pub kits: Vec<KitConfig>,
    pub paths: Vec<KitPath>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LinkedBinary {
    pub name: String,
    pub target: LinkTarget,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum LinkTarget {
    Default,
    Sdk { name: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KitConfig {
    pub name: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum SdkTarget {
    Dir { path: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KitPath {
    pub id: String,
    pub target: SdkTarget,
}

impl LocalConfig {
    pub fn graal_bins() -> Vec<String> {
        let out = vec![
            "native-image",
            "gu",
            "rebuild-images",
            "polyglot",
            "lli",
        ];

        let asvec = out.iter().map(|b| String::from(*b)).collect();
        return asvec;
    }

    pub fn node_bins() -> Vec<String> {
        let out = vec![
            "js",
            "node",
            "npx",
            "npm",
        ];

        let asvec = out.iter().map(|b| String::from(*b)).collect();
        return asvec;
    }

    pub fn jdk_bins() -> Vec<String> {
        let out = vec![
            "jar",
            "jarsigner",
            "java",
            "javac",
            "javadoc",
            "javap",
            "jcmd",
            "jconsole",
            "jdb",
            "jdeprscan",
            "jdeps",
            "jfr",
            "jhsdb",
            "jimage",
            "jinfo",
            "jjs",
            "jlink",
            "jmap",
            "jmod",
            "jps",
            "jrunscript",
            "jshell",
            "jstack",
            "jstat",
            "jstatd",
            "jvisualvm",
            "keytool",
            "pack200",
            "rmic",
            "rmid",
            "rmiregistry",
            "serialver",
            "unpack200",
        ];
        return out.iter().map(|b| String::from(*b)).collect();
    }

    pub fn example() -> LocalConfig {
        let example = LocalConfig {
            bins: vec![],
            kits: vec![],
            paths: vec![],
        };
        return example;
    }
}

impl GlobalConfig {
    pub fn example() -> GlobalConfig {
        let bins: Vec<(String, String)> = [
            LocalConfig::graal_bins().iter().map(|bin| (String::from(bin), String::from("jdk-graal"))).collect::<Vec<(String, String)>>(),
            LocalConfig::jdk_bins().iter().map(|bin| (String::from(bin), String::from("jdk"))).collect::<Vec<(String, String)>>(),
            LocalConfig::node_bins().iter().map(|bin| (String::from(bin), String::from("node"))).collect::<Vec<(String, String)>>(),
        ]
            .concat();

        let bins = bins.into_iter().map(|(name, sdk)| LinkedBinary { name: name, target: LinkTarget::Sdk { name: sdk } }).collect();


        let example = GlobalConfig {
            bins: bins,
            kits: vec![
                KitConfig { name: String::from("jdk"), id: String::from("graalvm") },
                KitConfig { name: String::from("jdk-graal"), id: String::from("graalvm") },
            ],
            paths: vec![
                KitPath { id: String::from("graalvm"), target: SdkTarget::Dir { path: String::from("/Library/Java/JavaVirtualMachines/graalvm-ce-java11-20.2.0/Contents/Home/bin/") } },
            ],
        };
        return example;
    }
}