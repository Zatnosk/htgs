use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[macro_use] extern crate nom;
use nom::{alphanumeric};

fn main(){
	let mut file = open_file();
	let mut buf: &mut Vec<u8> = &mut Vec::new();
	let _ = file.read_to_end(buf);
	let res = parser(buf.as_slice());

	println!("{:?}\n", res);
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
		id: identifier >>
		ws!(tag!("->")) >>
		cont: opt!(context) >>
		(Context::Reference(id,Box::new(cont)))
	) |
	do_parse!(id:identifier >> (Context::Identifier(id)))
));

named!(identifier, ws!(is_not!("(){}\" \t\n\r-><")));
named!(stringlit, delimited!(tag!("\""), is_not!("\""), tag!("\"")));
