# Hyper Text Generator Script / HTGS
a new way of generating HTML
```
html {
	head {
		meta (charset = utf-8)
		link (rel=stylesheet src=stylesheet.css)
		script (src=some-essential-external-framework-or-other.js)
		title {"HTML Generator Scripting Language Example"}
	}
	body {
		header {
			h1 {
				"A New Domain Specific Language"
				span {"a new way of generating HTML"}
			}
			nav = nav
		}
		main = main {
			p {"This is the main page explaining what's going on."}
		}
	}
}

nav -> a(href=#what){"What?"}
nav -> a(href=#why){"Why?"}
nav -> a("external" id=githublink href=https://github.com/zatnosk/htgs){"Fork on github"}
nav -> (class += "some-class other-class")

main -> {
	p {"This is some more content for the main element"}
}

/* This is a comment.
This is what's still missing:
	* functions generating larger structures,
		straight forward if element generating keywords is seen as functions
		user functions would be specified with a function keyword
		it might be useful to auto-map over list (of elements, of text, etc.)
		functions can take options inside parens (like elements) and "content" inside curly-parens
		element generators are just special predefined functions
		it might be useful to declare what context a function can be called from
			* element generators can be called from global or another element
			* SQL calls can be called from a database connection
	* some way of importing data from some external source,
		SQL calls where everything inside curly-parens is the SQL code
		parens might take options for connecting to database,
		or the SQL function might be called on a database "element" that's already connected
	* ifs and loops
*/

db-host = "localhost"
db-port = 9999
MySQL(username = "db-username" password = "db-password" host = db-host port = db-port){
	example1 = SQL{ SELECT * FROM example }
	otherdata = SQL{ SELECT * FROM `some-other-table`}
}
example2 = database->SQL{ SELECT * FROM example2}

for( row = example1 ){
	do-something-with { row }
	/* this is supposed to be a function invocation */
}
```

## Notes
### Use an AST based approach for SQL / database connections
See https://manowar.social/@zatnosk/188341 \
This language is basically a way to build tree structures, such as HTML. Treating SQL as another tree structure would fit in perfectly, although it might limit SQL use to features that this language is aware of; extra important that the language is easily extensible down to its core.
