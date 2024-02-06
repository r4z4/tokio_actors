use serde::{Deserialize, Serialize};
use url::Url;

// fn unbox_image_links(image_links: &str) -> &str {
//     image_links.trim_start_matches('[').trim_end_matches(']')
// }

// fn split_image_links(image_links_str: &str) -> Vec<&str> {
//     image_links_str.split(", ").collect()
// }

// fn remove_single_quotes(image_link: &str) -> &str {
//     image_link.trim_start_matches('\'').trim_end_matches('\'')
// }

// fn parse_image_link(image_link: &str) -> Option<Url> {
//     Url::parse(image_link).ok()
// }

pub fn parse_image_links(links_str: &str) -> Result<Option<Vec<Url>>, &'static str> {
    links_str
        .unbox_image_links()
        .split_image_links()
        .into_iter()
        .map(|link| link.remove_single_quotes().parse_image_link())
        .collect::<Result<Option<Vec<Url>>, _>>()
}

trait ParseImageLink {
    fn unbox_image_links(&self) -> &str;
    fn split_image_links(&self) -> Vec<&str>;
    fn remove_single_quotes(&self) -> &str;
    fn parse_image_link(&self) -> Result<Option<Url>, &'static str>;
}

impl ParseImageLink for &str {
    fn parse_image_link(&self) -> Result<Option<Url>, &'static str> {
        match self {
            &"No Images" => Ok(None),
            _ => Url::parse(self)
                .map(Some)            
                .map_err(|_| "Invalid Url"),
        }
    }
    fn remove_single_quotes(&self) -> &str {
        self.trim_start_matches('\'').trim_end_matches('\'')
    }
    fn split_image_links(&self) -> Vec<&str> {
        self.split(", ").collect()
    }
    fn unbox_image_links(&self) -> &str {
        self.trim_start_matches('[').trim_end_matches(']')
    }
}

pub fn deserialize_image_links<'de, D>(deserializer: D) -> Result<Option<Vec<Url>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let links_str: &str = Deserialize::deserialize(deserializer)?;
    parse_image_links(links_str).map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod tests {
    use super::*;
    const IMAGE_LINKS_EX: &str = "['NonUrlString','SecondRegularString']";

    #[test]
    fn test_unbox_image_links() {
        assert_eq!(IMAGE_LINKS_EX.unbox_image_links(), "'NonUrlString','SecondRegularString'");
    }

    // #[test]
    // fn test_split_image_links() {
    //     const IMAGE_LINKS_STR: &str = "'NonUrlString','SecondRegularString'";
    //     assert_eq!(IMAGE_LINKS_STR.split_image_links(), vec!["'NonUrlString'","'SecondRegularString'"]);
    // }
    
    // #[test]
    // fn test_parse_image_links() {
    //     const IMAGE_LINKS_TEST: &str = "['https://www.google.com/test','http://hello.com/photo.jpeg']";
    //     assert_eq!(parse_image_links(IMAGE_LINKS_TEST), Ok(Some(vec![
    //         Url::parse("https://www.google.com/test").unwrap(),
    //         Url::parse("http://hello.com/photo.jpeg").unwrap()
    //     ])));
    // }

    #[test]
    fn test_parse_image_links_invalid_url() {
        const IMAGE_LINKS_TEST: &str = "['No Images']";
        assert_eq!(parse_image_links(IMAGE_LINKS_TEST), Ok(None));
    }

    // Will this ever happen. If so, returning None probably not idea. Unlikely to happen, though.
    // #[test]
    // fn test_parse_image_links_invalid_url_and_valid() {
    //     const IMAGE_LINKS_TEST: &str = "['No Images', 'http://hello.com/photo.jpeg']";
    //     assert_eq!(parse_image_links(IMAGE_LINKS_TEST), None);
    // }
}