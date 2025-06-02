use std::path::Path;
use tantivy::collector::{Count, TopDocs};
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexWriter, ReloadPolicy};
use tantivy::directory::MmapDirectory;
use tantivy::TantivyDocument;
use anyhow::Result;

// Define a struct for our document for easier handling
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct SearchableDocument {
    pub id: String,
    pub title: String,
    pub body: String,
    pub source: String, // e.g., "rust-docs", "api-spec-v1"
    pub version: Option<String>, // Optional versioning
}

pub struct SearchService {
    pub index: Index,
    pub schema: Schema,
    // Fields for the schema
    pub id_field: Field,
    pub title_field: Field,
    pub body_field: Field,
    pub source_field: Field,
    pub version_field: Field,
}

impl SearchService {
    pub fn new(index_path: impl AsRef<Path>) -> Result<Self> {
        let mut schema_builder = Schema::builder();
        let id_field = schema_builder.add_text_field("id", STRING | STORED); // Unique ID for the document
        let title_field = schema_builder.add_text_field("title", TEXT | STORED);
        let body_field = schema_builder.add_text_field("body", TEXT | STORED); // Main content for full-text search
        let source_field = schema_builder.add_text_field("source", STRING | STORED | FAST); // Faceting/filtering
        let version_field = schema_builder.add_text_field("version", STRING | STORED | FAST); // Optional, for filtering

        let schema = schema_builder.build();
        
        let index_dir = index_path.as_ref();
        std::fs::create_dir_all(index_dir)?; // Ensure directory exists

        let directory = MmapDirectory::open(index_dir)?;
        let index = Index::open_or_create(directory, schema.clone())?;

        Ok(SearchService {
            index,
            schema,
            id_field,
            title_field,
            body_field,
            source_field,
            version_field,
        })
    }

    pub fn add_document(&self, doc_to_add: SearchableDocument, writer_mem_budget: usize) -> Result<()> {
        // Create an IndexWriter. Consider managing this more globally or per-batch for performance.
        // For simplicity here, we create one per add.
        // The memory budget is per thread.
        let mut index_writer: IndexWriter = self.index.writer(writer_mem_budget)?; 

        // Create the document with base fields
        let doc = doc!(
            self.id_field => doc_to_add.id.clone(),
            self.title_field => doc_to_add.title.clone(),
            self.body_field => doc_to_add.body.clone(),
            self.source_field => doc_to_add.source.clone()
        );
        
        // Add version if present - fix: first modify doc, then add it
        if let Some(version) = &doc_to_add.version {
            // Fix: Dereference the version string to avoid double reference
            index_writer.add_document(doc!(
                self.id_field => doc_to_add.id.clone(),
                self.title_field => doc_to_add.title.clone(),
                self.body_field => doc_to_add.body.clone(),
                self.source_field => doc_to_add.source.clone(),
                self.version_field => version.clone() // Clone the String to pass it by value
            ))?;
        } else {
            index_writer.add_document(doc)?;
        }
        
        index_writer.commit()?; // Committing makes changes visible
        println!("Document added and committed: {}", doc_to_add.id);
        Ok(())
    }

    pub fn search_documents(&self, query_str: &str, limit: usize) -> Result<Vec<SearchableDocument>> {
        let reader = self.index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual) // Or OnCommit
            .try_into()?;

        let searcher = reader.searcher();
        let query_parser = QueryParser::for_index(&self.index, vec![self.title_field, self.body_field]);
        let query = query_parser.parse_query(query_str)?;

        let top_docs = searcher.search(&query, &(TopDocs::with_limit(limit), Count))?;
        
        let mut results = Vec::new();
        for (_score, doc_address) in top_docs.0 {
            // Use the correct type parameter with searcher.doc()
            let retrieved_doc = searcher.doc::<TantivyDocument>(doc_address)?;
            
            // Fix: use appropriate methods to extract text values
            let id = retrieved_doc.get_first(self.id_field)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
                
            let title = retrieved_doc.get_first(self.title_field)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
                
            let body = retrieved_doc.get_first(self.body_field)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
                
            let source = retrieved_doc.get_first(self.source_field)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
                
            let version = retrieved_doc.get_first(self.version_field)
                .and_then(|v| v.as_str())
                .map(String::from);
                
            results.push(SearchableDocument {
                id,
                title,
                body,
                source,
                version,
            });
        }
        Ok(results)
    }
} 