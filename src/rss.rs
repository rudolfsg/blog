use rss::{ChannelBuilder, Item};
use crate::post::Post;

pub fn generate_rss(posts: &Vec<Post>, blog_name: &str, blog_url: &str) -> String {
    let mut channel = ChannelBuilder::default()
    .title(blog_name.to_string())
    .link(blog_url.to_string())
    .description("blog".to_string())
    .build();

    let mut items: Vec<Item> = Vec::new();

    for post in posts {
        let mut item = Item::default();
        item.set_title(post.metadata.title.clone());
        item.set_pub_date(post.metadata.date.to_string()); 
        item.set_link(post.metadata.url.clone());
        items.push(item); 
    }
    channel.set_items(items); 

    let xml = channel.to_string();
    xml
}