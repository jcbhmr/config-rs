use glob::Pattern;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Error {
    MoreThanFourComponents,
    BlankOsOnlyAllowedWithExplicitMachineCodeFileFormat,
    OsNotRecognized(String),
    MachineCodeFormatNotRecognized(String),
    CpuIsNotValidWithOs(String, String),
    LibcNeedsExplicitKernel(String),
    OsNeedsExplicitKernel(String),
    KernelDoesNotSupportOs(String, String),
    OsNeedsWindows(String),
    KernelNotKnownToWorkWithOs(String, String),
}

macro_rules! matches_glob {
    ($input:expr, $pattern:expr) => {
        Pattern::new($pattern).expect("pattern should be valid").matches($input)
    };
}

pub fn config_sub(input: impl AsRef<str>) -> Result<String, Error> {
    fn inner(input: &str) -> Result<String, Error> {
        if input.contains("local") {
            // TODO: Verify format (no spaces, alphanumeric, etc.)
            return Ok(input.into());
        }

        let fields = input.split('-').collect::<Vec<_>>();
        
        let (basic_machine, basic_os): (String, String) = match fields.len() {
            _ => return Err(Error::MoreThanFourComponents),
            4 => (format!("{}-{}", fields[0], fields[1]), format!("{}-{}", fields[2], fields[3])),
            3 => match (fields[1], fields[2]) {
                (a, b) if 
                    (a.starts_with("cloudabi") && b.starts_with("eabi"))
                    || (a.starts_with("kfreebsd") && b.starts_with("gnu"))
                    || (a.starts_with("knetbsd") && b.starts_with("gnu"))
                    || (a.starts_with("kopensolaris") && b.starts_with("gnu"))
                    || a == "ironclad"
                    || a == "linux"
                    || a == "managarm"
                    || (a.starts_with("netbsd") && b.starts_with("eabi"))
                    || (a.starts_with("netbsd") && b.starts_with("gnu"))
                    || (a == "nto" && b.starts_with("qnx"))
                    || (a == "os2" && b.starts_with("emx"))
                    || (a == "rtmk" && b.starts_with("nova"))
                    || (a == "storm" && b.starts_with("chaos"))
                    || (a == "uclinux" && b.starts_with("gnu"))
                    || (a == "uclinux" && b.starts_with("uclibc"))
                    || (a == "windows")
                    => (fields[0].into(), format!("{}-{}", fields[1], fields[2])),
                ("android", "linux") => (format!("{}-unknown", fields[0]), "linux-android".into()),
                _ => (format!("{}-{}", fields[0], fields[1]), fields[2].into()),
            },
            2 => match (fields[0], fields[1]) {
                ("convex", b) if matches_glob!(b, "c[12]") || matches_glob!(b, "c3[248]") => (format!("{}-convex", fields[1]), "".into()),
                ("decstation", "3100") => ("mips-dec".into(), "".into()),
                _ => match fields[1] {
                    s if matches_glob!(s, "sun*os*") => (fields[0].into(), fields[1].into()),
                    s if
                        s.starts_with("3100")
                        || s.starts_with("32")
                        || s.starts_with("3300")
                        || s.starts_with("3600")
                        || s.starts_with("7300")
                        || s == "acorn"
                        || s.starts_with("altos")
                        || s == "apollo"
                        || s == "apple"
                        || s == "atari"
                        || s.starts_with("att")
                        || s == "axis"
                        || s == "be"
                        || s == "bull"
                        || s == "cbm"
                        || s == "ccur"
                        || s == "cisco"
                        || s == "commodore"
                        || s.starts_with("convergent")
                        || s.starts_with("convex")
                        || s == "cray"
                        || s == "crds"
                        || s.starts_with("dec")
                        || s.starts_with("delta")
                        || s == "dg"
                        || s == "digital"
                        || s == "dolphin"
                        || s.starts_with("encore")
                        || s == "gould"
                        || s == "harris"
                        || s == "highlevel"
                        || s.starts_with("hitachi")
                        || s == "hp"
                        || s.starts_with("ibm")
                        || s == "intergraph"
                        || s.starts_with("isi")
                        || s == "knuth"
                        || s == "masscomp"
                        || s.starts_with("microblaze")
                        || s.starts_with("mips")
                        || s.starts_with("motorola")
                        || s.starts_with("ncr")
                        || s == "news"
                        || s == "next"
                        || s == "ns"
                        || s == "oki"
                        || s.starts_with("omron")
                        || s.starts_with("pc533")
                        || s == "rebel"
                        || s == "rom68k"
                        || s == "rombug"
                        || s == "semi"
                        || s.starts_with("sequent")
                        || s.starts_with("sgi")
                        || s == "siemens"
                        || s == "sim"
                        || s == "sni"
                        || s.starts_with("sony")
                        || s == "stratus"
                        || s == "sun"
                        || matches_glob!(s, "sun[234]*")
                        || s == "tektronix"
                        || s.starts_with("tti")
                        || s == "ultra"
                        || s.starts_with("unicom")
                        || s == "wec"
                        || s == "winbond"
                        || s == "wrs"
                        => (format!("{}-{}", fields[0], fields[1]), "".into()),
                    s if s.starts_with("tock") || s.starts_with("zephyr") => (format!("{}-unknown", fields[0]), fields[1].into()),
                    _ => (fields[0].into(), fields[1].into()),
                }
            }
            1 => match fields[0] {
                "386bsd" => ("i386-pc".into(), "bsd".into()),
                _ => todo!(),
            },
            0 => panic!("input should have at least one field"),
        };

        let (cpu, mut vendor): (String, String) = match basic_machine.as_str() {
            "w89k" => ("hppa1.1".into(), "winbond".into()),
            _ => todo!(),
            s if matches_glob!(s, "i*86") || s == "x86_64" => (s.into(), "pc".into()),
            "pc98" => ("i386".into(), "pc".into()),
            "x64" | "amd64" => ("x86_64".into(), "pc".into()),
            s => s.split_once('-').map(|(a, b)| (a.into(), b.into())).unwrap_or_else(|| (s.into(), "unknown".into())),
        };

        drop(basic_machine);

        match vendor.as_str() {
            s if s.starts_with("digital") => vendor = "dec".into(),
            s if s.starts_with("commodore") => vendor = "cbm".into(),
            _ => {},
        };

        let (kernel, os, obj): (String, String, String) = if basic_os != "" {
            let mut obj: String = "".into();

            let (kernel, mut os): (String, String) = match basic_os.as_str() {
                s if s.starts_with("gnu/linux") => ("linux".into(), s.replace("gnu/linux", "gnu").into()),
                "os2-emx" => ("os2".into(), "emx".into()),
                s if s.starts_with("nto-qnx") => ("nto".into(), s.replace("nto-qnx", "qnx").into()),
                s if s.starts_with("nto") => ("nto".into(), s.replace("nto", "qnx").into()),
                s if s.starts_with("ironclad") => ("ironclad".into(), s.replace("ironclad", "mlibc").into()),
                s if s.starts_with("linux") => ("linux".into(), s.replace("linux", "gnu").into()),
                s if s.starts_with("managarm") => ("managarm".into(), "mlibc".into()),
                s => s.split_once('-').map(|(a, b)| (a.into(), b.into())).unwrap_or_else(|| ("".into(), s.into())),
            };

            match os.as_str() {
                "auroraux" => os = "auroraux".into(),
                s if s.starts_with("bluegene") => os = "cnk".into(),
                s if s == "solaris1" || s.starts_with("solaris1.") => os = os.replace("solaris1", "sunos4").into(),
                "solaris" => os = "solaris2".into(),
                s if s.starts_with("unixware") => os = "sysv4.2uw".into(),
                "ns" | "ns1" | "nextstep" | "nextstep1" | "openstep1" => os = "nextstep".into(),
                "ns2" | "nextstep2" | "openstep2" => os = "nextstep2".into(),
                "ns3" | "nextstep3" | "openstep" | "openstep3" => os = "openstep3".into(),
                "ns4" | "nextstep4" | "openstep4" => os = "openstep4".into(),
                s if s.starts_with("es1800") => os = "ose".into(),
                s if s.starts_with("chorusos") => os = "chorusos".into(),
                "isc" => os = "isc2.2".into(),
                "sco6" => os = "sco5v6".into(),
                "sco5" => os = "sco3.2v5".into(),
                "sco4" => os = "sco3.2v4".into(),
                s if matches_glob!(s, "sco3.2.[4-9]*") => os = s.replace("sco3.2.", "sco3.2v").into(),
                s if matches_glob!(s, "sco*v*") || s == "scout" => {},
                s if s.starts_with("sco") => os = "sco3.2v2".into(),
                s if s.starts_with("psos") => os = "psos".into(),
                _ => todo!(),
                s if s.starts_with("pikeos") => match cpu.as_str() {
                    s if s.starts_with("arm") => os = "eabi".into(),
                    _ => {
                        os = "".into();
                        obj = "elf".into();
                    },
                },
                s if s.starts_with("aout") || s.starts_with("coff") || s.starts_with("elf") || s.starts_with("pe") => {
                    obj = os.clone();
                    os = "".into();
                },
                _ => {},
            };

            (kernel, os, obj)
        } else {
            todo!()
        };

        match os.as_str() {
            s if
                s.starts_with("llvm")
                || s.starts_with("musl")
                || s.starts_with("newlibc")
                || s.starts_with("relibc")
                || s.starts_with("uclibc")
                => {},
            s if s.starts_with("eabi") || s.starts_with("gnueabi") => {}
            "simlinux" | "simwindows" | "spe" => {},
            "ghcjs" => {},
            _ => todo!(),
            "uefi" => {},
            "none" => {},
            s if s.starts_with("kernel") || s.starts_with("msvc") => {},
            "" => {
                if obj == "" {
                    return Err(Error::BlankOsOnlyAllowedWithExplicitMachineCodeFileFormat);
                }
            },
            _ => {
                return Err(Error::OsNotRecognized(os));
            }
        };

        match obj.as_str() {
            s if s.starts_with("aout") || s.starts_with("coff") || s.starts_with("elf") || s.starts_with("pe") => {},
            "" => {},
            _ => {
                return Err(Error::MachineCodeFormatNotRecognized(obj));
            }
        };

        match (cpu.as_str(), os.as_str()) {
            ("javascript", "ghcjs") => {},
            ("javascript", _) | (_, "ghcjs") => {
                return Err(Error::CpuIsNotValidWithOs(cpu, os));
            },
            _ => {},
        };

        match (cpu.as_str(), os.as_str(), obj.as_str()) {
            ("linux", s, "") if
                s.starts_with("gnu")
                || s.starts_with("android")
                || s.starts_with("dietlibc")
                || s.starts_with("llvm")
                || s.starts_with("mlibc")
                || s.starts_with("musl")
                || s.starts_with("newlib")
                || s.starts_with("relibc")
                || s.starts_with("uclibc")
                || s.starts_with("ohos")
             => {},
            ("uclinux", s, "") if s.starts_with("uclibc") || s.starts_with("gnu") => {},
            ("ironclad", s, "") if s.starts_with("mlibc") => {},
            _ => todo!(),
            ("", s, "") if s.starts_with("kernel") => return Err(Error::OsNeedsExplicitKernel(os)),
            (_, s, "") if s.starts_with("kernel") => return Err(Error::KernelDoesNotSupportOs(kernel, os)),
            (_, s, "") if s.starts_with("msvc") => return Err(Error::OsNeedsWindows(os)),
            _ => todo!(),
            ("none", "", _) => {},
            ("", _, "") => {},
            ("", "", _) => {},
            (_, _, _) => return Err(Error::KernelNotKnownToWorkWithOs(kernel, os)),
        };

        if vendor == "unknown" {
            match (cpu.as_str(), os.as_str()) {
                _ => todo!(),
            };
        }

        Ok(format!(
            "{}-{}{}{}{}",
            cpu,
            vendor,
            if kernel == "" { "".into() } else { format!("-{}", kernel) },
            if os == "" { "".into() } else { format!("-{}", os) },
            if obj == "" { "".into() } else { format!("-{}", obj) },
        ))
    }
    inner(input.as_ref())
}
