# entity-generator

`entity-generator` is a command-line tool written in Rust that reads your Prisma schema file and automatically generates TypeScript classes for mappers, entities, and repositories. This tool is designed to streamline the process of setting up a structured data layer in your TypeScript applications, saving time and reducing the need for repetitive coding.

## Features

- Parses your Prisma schema file.
- Generates TypeScript classes for:
  - **Mapper**: Handles data transformations.
  - **Entity**: Represents your data models.
  - **Repository**: Manages database operations.

## Installation

Download the latest release from [Releases](https://github.com/xsadia/entity_generator/releases) and add the binary to your system's PATH.

## Usage

Run the following command in the root of your project and choose the model you want to create an entity, mapper or repository of and choose the output module:

```
entity_generator
```

# Demo

https://github.com/user-attachments/assets/45d9cb91-b804-4afd-bd2f-42fb0f43d5a4
