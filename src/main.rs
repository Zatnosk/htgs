use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::collections::HashMap;

#[macro_use] extern crate nom;
use nom::{alphanumeric,IResult};

fn main(){
	let mut file = open_file();
	let mut buf: &mut Vec<u8> = &mut Vec::new();
	let _ = file.read_to_end(buf);

	let res = match parser(buf.as_slice()){
		IResult::Done(_, ast) => parse_exprs(ast),
		IResult::Error(_) => Vec::new(),
		IResult::Incomplete(_) => Vec::new()
	};

	println!("\n{:?}\n", res);
}

fn open_file() -> std::fs::File{
    let path = Path::new("test.htgs");
    let display = path.display();

    let file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why.description()),
        Ok(file) => file,
    };

    return file;
}

fn parse_exprs(ast: Vec<Expr>) -> Vec<HContent>{
	let mut content = Vec::new();
	for e in ast {
		let elem = match e {
			Expr::ElemFull(id, attrs, content) => HContent::Node(HElement{
				tagname: parse_context(id),
				attributes: parse_attrs(attrs),
				content: parse_exprs(content)
			}),
			Expr::ElemSlim(id, content) => HContent::Node(HElement{
				tagname: parse_context(id),
				attributes: HashMap::new(),
				content: parse_exprs(content)
			}),
			Expr::ElemEmpty(id, attrs) => HContent::Node(HElement{
				tagname: parse_context(id),
				attributes: parse_attrs(attrs),
				content: Vec::new()
			}),
			Expr::ElemSlimEmpty(id) => HContent::Node(HElement{
				tagname: parse_context(id),
				attributes: HashMap::new(),
				content: Vec::new()
			}),
			Expr::Str(s) => HContent::Text(String::from_utf8_lossy(s).into_owned()),
		};
		content.push(elem);
	}
	return content;
}

fn parse_attrs(attrs: AttrList) -> HashMap<String, Vec<String>>{
	let mut map = HashMap::new();
	match attrs.s {
		Some(value) => {
			map.insert(
				String::from("$"),
				vec!(String::from_utf8_lossy(value).into_owned())
			);
		},
		None => ()
	}
	for attr in attrs.attrs {
		match attr {
			Attr::Key(key) =>{
				map.insert(
					String::from_utf8_lossy(key).into_owned(),
					Vec::new()
				);
			},
			Attr::KeyValue(key,value) => {
				map.insert(
					String::from_utf8_lossy(key).into_owned(),
					vec!(String::from_utf8_lossy(value).into_owned())
				);
			},
			Attr::KeyValueAdd(k,value) => {
				let key = String::from_utf8_lossy(k).into_owned();
				map.entry(key)
					.or_insert_with(Vec::new)
					.push(String::from_utf8_lossy(value).into_owned());
			}
		}
	}
	return map;
}

fn parse_context(id: Context) -> String {
	let string = match id {
		// continue work here
		Context::Assignment(_,_) => String::new(),
		Context::Identifier(s) => String::from_utf8_lossy(s).into_owned(),
		Context::Reference(_,_) => String::new()
	};
	return string;
}

#[derive(Debug)]
struct HDocument {
	content: Vec<HContent>
}

#[derive(Debug)]
enum HContent {
	Text(String),
	Node(HElement)
}

#[derive(Debug)]
struct HElement {
	tagname: String,
	attributes: HashMap<String, Vec<String>>,
	content: Vec<HContent>
}

#[derive(Debug)]
enum Expr<'a> {
	Str(&'a [u8]),
	ElemFull(Context<'a>, AttrList<'a>, Vec<Expr<'a>>),
	ElemEmpty(Context<'a>, AttrList<'a>),
	ElemSlim(Context<'a>, Vec<Expr<'a>>),
	ElemSlimEmpty(Context<'a>)
}

#[derive(Debug)]
enum Attr<'a> {
	KeyValueAdd(&'a [u8],&'a [u8]),
	KeyValue(&'a [u8],&'a [u8]),
	Key(&'a [u8])
}

#[derive(Debug)]
struct AttrList<'a> {
	s: Option<&'a [u8]>,
	attrs: Vec<Attr<'a>>
}

#[derive(Debug)]
enum Context<'a> {
	Assignment(&'a [u8], Box<Context<'a>>),
	Reference(&'a [u8], Box<Option<Context<'a>>>),
	Identifier(&'a [u8])
}


named!(parser<&[u8], Vec<Expr> >, many1!(expression));
named!(expression<&[u8], Expr>, alt!(element | string));

named!(string<&[u8], Expr>, do_parse!( s: stringlit >> (Expr::Str(s)) ));

named!(element<&[u8], Expr>,
	alt_complete!( element_full | element_empty | element_slim | element_slim_empty )
);

named!(element_full<&[u8], Expr>,
	do_parse!( id: context >> opt: options >> body: body >> (Expr::ElemFull(id,opt,body)) )
);
named!(element_empty<&[u8], Expr>,
	do_parse!( id: context >> opt: options >> (Expr::ElemEmpty(id,opt)) )
);
named!(element_slim<&[u8], Expr>,
	do_parse!( id: context >> body: body >> (Expr::ElemSlim(id,body)) )
);
named!(element_slim_empty<&[u8], Expr>,
	do_parse!( id: context >> (Expr::ElemSlimEmpty(id)) )
);

named!(body<&[u8], Vec<Expr> >, ws!(delimited!(tag!("{"), many0!(expression), tag!("}"))));

named!(options<&[u8], AttrList>,
	ws!(delimited!(tag!("("),
		do_parse!(
			s: opt!(stringlit) >>
			attrs: many0!(attribute) >>
			(AttrList{s: s, attrs: attrs})
		),
	tag!(")")))
);

named!(attribute<&[u8], Attr>,
	alt!(
		do_parse!(
			key: alphanumeric >>
			ws!(tag!("=")) >>
			value: alt!(stringlit | is_not!("(){}\" \t\n\r")) >>
			(Attr::KeyValue(key, value))
		) |
		do_parse!(
			key: alphanumeric >>
			ws!(tag!("+=")) >>
			value: alt!(stringlit | is_not!("(){}\" \t\n\r")) >>
			(Attr::KeyValueAdd(key, value))
		) |
		do_parse!(key: alphanumeric >> (Attr::Key(key)))
	)
);

named!(context<&[u8], Context>, alt!(
	do_parse!(
		lh: identifier >>
		ws!(tag!("=")) >>
		rh: context >>
		(Context::Assignment(lh,Box::new(rh)))
	) |
	do_parse!(
		id: identifier >>
		ws!(tag!("->")) >>
		cont: opt!(context) >>
		(Context::Reference(id,Box::new(cont)))
	) |
	do_parse!(id:identifier >> (Context::Identifier(id)))
));

named!(identifier, ws!(is_not!("(){}\" \t\n\r-><")));
named!(stringlit, delimited!(tag!("\""), is_not!("\""), tag!("\"")));
