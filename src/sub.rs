use regex_lite::Regex;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, thiserror::Error)]
pub enum Error {
    #[error("more than four components")]
    MoreThanFourComponents,
}

/// Canonicalize a configuration name
pub fn sub(cpu_mfr_opsys_or_alias: &str) -> Result<String, Error> {
    // Split fields of configuration type
    let (field1, field2, field3, field4) = {
        let iter = cpu_mfr_opsys_or_alias.splitn(4, "-");
        let field1 = iter.next();
        let field2 = iter.next();
        let field3 = iter.next();
        let field4 = iter.next();
        (field1, field2, field3, field4)
    };

    // Separate into logical components for further validation
    let (basic_machine, basic_os) = match cpu_mfr_opsys_or_alias {
        x if Regex::new("^(.*?)-(.*?)-(.*?)-(.*?)-(.*?)$")
            .unwrap()
            .is_match(x) =>
        {
            return Err(SubError::MoreThanFourComponents);
        }
        x if Regex::new("^(.*?)-(.*?)-(.*?)-(.*?)$").unwrap().is_match(x) => (
            format!("{}-{}", field1.unwrap(), field2.unwrap()),
            format!("{}-{}", field3.unwrap(), field4.unwrap()),
        ),
        x if Regex::new("^(.*?)-(.*?)-(.*?)$").unwrap().is_match(x) => {
            // Ambiguous whether COMPANY is present, or skipped and KERNEL-OS is two parts
            let maybe_os = format!("{}-{}", field2.unwrap(), field3.unwrap());
            match maybe_os.as_ref() {
                x if Regex::new("^cloudabi(.*?)-eabi(.*?)$").unwrap().is_match(x)
                | Regex::new("^kfreebsd(.*?)-gnu(.*?)$").unwrap().is_match(x)
                | Regex::new("^knetbsd(.*?)-gnu(.*?)$").unwrap().is_match(x)
                | Regex::new("^kopensolaris(.*?)-gnu(.*?)$").unwrap().is_match(x)
                | Regex::new("^ironclad-(.*?)$").unwrap().is_match(x)
                | Regex::new("^linux-(.*?)$").unwrap().is_match(x)
                | Regex::new("^managarm-(.*?)$").unwrap().is_match(x)
                | Regex::new("^netbsd(.*?)-eabi(.*?)$").unwrap().is_match(x)
                | Regex::new("^netbsd(.*?)-gnu(.*?)$").unwrap().is_match(x)
                | Regex::new("^nto-qnx(.*?)$").unwrap().is_match(x)
                | Regex::new("^os2-emx(.*?)$").unwrap().is_match(x)
                | Regex::new("^rtmk-nova(.*?)$").unwrap().is_match(x)
                | Regex::new("^storm-chaos(.*?)$").unwrap().is_match(x)
                | Regex::new("^uclinux-uclibc(.*?)$").unwrap().is_match(x)
                | Regex::new("^windows-(.*?)$").unwrap().is_match(x) => (field1.unwrap().to_owned(), maybe_os.to_owned()),
                "android-linux" => (format!("{}-unknown", field1.unwrap()), "linux-android".to_owned()),
                _ => (format!("{}-{}", field1.unwrap(), field2.unwrap()), field3.unwrap().to_owned()),
            }
        },
        x if Regex::new("^(.*?)-(.*?)$").unwrap().is_match(x) => {
            match format!("{}-{}", field1.unwrap(), field2.unwrap()).as_ref() {
                // Shorthands that happen to contain a single dash
                x if Regex::new("^convex-c[12]$").unwrap().is_match(x)
                | Regex::new("^convex-c3[248]$").unwrap().is_match(x) => (format!("{}-convex", field1.unwrap()), "".to_owned()),
                "decstation-3100" => ("mips-dec".to_owned(), "".to_owned()),
                x if Regex::new("^(.*?)-(.*?)$").unwrap().is_match(x) => {
                    // Second component is usually, but not always, the OS
                    match field2.unwrap() {
                        // Do not treat sunos as manufacturer
                        x if Regex::new("^sun(.*?)os(.*?)$").unwrap().is_match(x) => (field1.unwrap().to_owned(), field2.unwrap().to_owned()),
                        // Manufacturers
                        x if Regex::new("^3100(.*?)$").unwrap().is_match(x)
                        | Regex::new("^32(.*?)$").unwrap().is_match(x)
                        | Regex::new("^3300(.*?)$").unwrap().is_match(x)
                        | Regex::new("^3600(.*?)$").unwrap().is_match(x)
                        | Regex::new("^7300(.*?)$").unwrap().is_match(x)
                        | "acorn"
                        | Regex::new("^altos(.*?)$").unwrap().is_match(x) => todo!()
                    }
                }
            }
        }
    };
}

fn separate_into_logical_components_for_further_validation() {

}

fn decode_1_component_or_ad_hoc_basic_machines() {}

fn decode_basic_machines_in_the_full_and_proper_cpu_company_form() {}

fn canonicalize_certain_aliases_for_manufacturers() {}

fn decode_manufacturer_specific_aliases_for_certain_operating_systems() {}

fn recognize_some_ad_hoc_cases_or_perhaps_split_kernel_os_or_else_just_set_os() {}

fn normalize_the_os_knowing_we_just_have_one_component_its_not_a_kernel_etc() {}

