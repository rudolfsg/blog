use rss::{ChannelBuilder, Item, Guid};
use crate::post::Post;
use chrono::{TimeZone, Datelike};

pub fn generate_rss(posts: &Vec<Post>, blog_name: &str, blog_url: &str) -> String {

    let mut channel = ChannelBuilder::default()
    .title(blog_name.to_string())
    .link(blog_url.to_string())
    .description("blog".to_string())
    .build();

    let mut items: Vec<Item> = Vec::new();

    for post in posts {
        // rss feeds require rfc2822 format
        let dt = post.metadata.date; 
        let dt = chrono::Utc.with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0);
        let dt = dt.unwrap();

        let mut item = Item::default();
        item.set_title(post.metadata.title.clone());
        item.set_pub_date(dt.to_rfc2822()); 
        item.set_link(post.metadata.url.clone());
        // unique id for each post across the site
        let mut guid = Guid::default();
        guid.set_value(&post.metadata.slug);
        item.set_guid(guid);
        items.push(item); 
    }
    channel.set_items(items); 

    let xml = channel.to_string();
    xml
}