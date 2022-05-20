---
title: I made minimal change to escape HTML in REST API using go reflection
description: This blog post explains about go reflection and some of it's usecases
slug: i-made-minimal-change-to-escape-html-in-rest-using-go-reflection
authors:
  - name: Poonai
    title: Maintainer of inspektor
    url: https://twitter.com/poonai_
    image_url: https://i.imgur.com/RNM7R6Q.jpg
tags: [go,reflection]
image: /img/reflection.jpg
hide_table_of_contents: false
---
![reflection](/img/reflection.jpg)

To avoid SQL injection or storing raw HTML in the database, it's common practise to escape all SQL statements and HTML from the request body. As a result, when that raw HTML fetched from then database get displayed on the client side will not be executed.

Even OWASP(Open Web Application Security Project) suggests escaping HTML and SQL statement to secure the API. 

However, it is time-consuming to write a function for each request struct to escape each field. So I came up with the idea of creating a single function called `EscapeStruct` that uses reflection to detect the layout of the struct and then it can be used to escape all of the string fields.


Reflection in Golang allows you to inspect and manipulate the structure at runtime. The `reflect` package contains all functions related to reflection.

Let's walk through the implementation of `EscapeStruct` which I used to escape all struct string fields.

```go
func EscapeStruct(in interface{}) {
	reflectStruct := reflect.ValueOf(in).Elem()
	escapeValue(reflectStruct)
}
```

The EscapeStruct function accepts an interface as an argument, allowing us to pass any struct pointer as an argument. For the purposes of this blog post, assume that we will only pass a pointer to a struct as an argument, so that we can ignore all edge cases and emphasis on the solution's core.

`reflect.ValueOf` returns the `reflect.Value`, which contains the concrete value of the interface we passed. The `Elem` method will return the `reflect.Value` of the struct to which the given pointer points.

Now we have the `reflect.Value` of the underlying struct to which the given pointer points. This is later passed to the `esacpeValue` function. 


```go
func escapeValue(in reflect.Value) {
	if in.Kind() == reflect.Struct {
		n := in.NumField()
		for i := 0; i < n; i++ {
			field := in.Field(i)
			escapeValue(field)
		}
	}

	if in.Kind() == reflect.Ptr {
		escapeValue(in.Elem())
		return
	}

	if in.Kind() == reflect.String {
		if in.CanSet() {
			in.SetString(html.EscapeString(in.String()))
		}
		return
	}
}
```

`esacpeValue` function takes `reflect.Value` as an argument handles three cases. if the given argument
- is of kind struct, we'll iterate through the fields of the struct and pass the fields to the `escapeValue` function 
- is of pointer, we'll get the underlying object which the pointer points to using `Elem` method and then it get passed to `escapeValue` function. 
- is of string type, we'll check whether it can be mutated or not using `CanSet` method because private fields can't be mutated. If string field can be mutaed, then we set escaped value using `SetString` method. 

Because we are passing the fields of the struct back to the `escapeValue` function, the string fields in deep nested structs are checked recursively.
