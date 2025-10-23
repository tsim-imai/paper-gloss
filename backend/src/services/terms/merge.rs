use crate::models::{Definition, Occurrence, Term, TermVariant};
use anyhow::{Context, Result};
use sqlx::SqlitePool;

/// Term merge service
///
/// Merges source_term into target_term:
/// - Transfers all variants from source to target
/// - Updates all occurrences to point to target
/// - Optionally merges definitions
/// - Deletes source term
pub struct TermMergeService {
    pool: SqlitePool,
}

impl TermMergeService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Merge source term into target term
    ///
    /// # Arguments
    /// * `source_id` - ID of term to be merged (will be deleted)
    /// * `target_id` - ID of term to merge into (will be kept)
    /// * `confirmed` - User confirmation required
    ///
    /// # Returns
    /// Number of entities affected (variants + occurrences)
    pub async fn merge_terms(
        &self,
        source_id: &str,
        target_id: &str,
        confirmed: bool,
    ) -> Result<i64> {
        if !confirmed {
            anyhow::bail!("Merge operation requires user confirmation");
        }

        if source_id == target_id {
            anyhow::bail!("Cannot merge a term with itself");
        }

        // Verify both terms exist
        let source = Term::find_by_id(&self.pool, source_id)
            .await
            .context("Source term not found")?;
        let _target = Term::find_by_id(&self.pool, target_id)
            .await
            .context("Target term not found")?;

        let mut affected_count = 0;

        // 1. Transfer variants
        let source_variants = TermVariant::find_by_term_id(&self.pool, source_id).await?;
        for variant in &source_variants {
            // Check if target already has this variant
            let target_variants = TermVariant::find_by_term_id(&self.pool, target_id).await?;
            let already_exists = target_variants
                .iter()
                .any(|v| v.lang == variant.lang && v.surface == variant.surface);

            if !already_exists {
                // Create variant for target
                TermVariant::create(&self.pool, target_id.to_string(), variant.lang.clone(), variant.surface.clone()).await?;
                affected_count += 1;
            }

            // Delete source variant
            TermVariant::delete(&self.pool, &variant.id).await?;
        }

        // 2. Transfer occurrences
        let source_occurrences = Occurrence::find_by_term_id(&self.pool, source_id).await?;
        for occurrence in &source_occurrences {
            // Update occurrence to point to target term
            sqlx::query(
                r#"
                UPDATE occurrences
                SET term_id = ?
                WHERE id = ?
                "#,
            )
            .bind(target_id)
            .bind(&occurrence.id)
            .execute(&self.pool)
            .await?;
            affected_count += 1;
        }

        // 3. Handle definitions
        // If source has definition but target doesn't, transfer it
        let source_def = Definition::find_by_term_id(&self.pool, source_id).await?;
        let target_def = Definition::find_by_term_id(&self.pool, target_id).await?;

        if source_def.is_some() && target_def.is_none() {
            if let Some(def) = &source_def {
                // Create definition for target
                Definition::create(&self.pool, target_id.to_string(), def.text.clone(), def.provider.clone()).await?;
                affected_count += 1;
            }
        }

        // Delete source definition (if exists)
        if let Some(def) = source_def {
            Definition::delete(&self.pool, &def.term_id).await?;
        }

        // 4. Delete source term
        Term::delete(&self.pool, source_id).await?;

        tracing::info!(
            "Merged term '{}' ({}) into '{}' ({}), affected {} entities",
            source.lemma_en,
            source_id,
            _target.lemma_en,
            target_id,
            affected_count
        );

        Ok(affected_count)
    }

    /// Preview merge impact without executing
    ///
    /// Returns (variants_to_transfer, occurrences_to_transfer, has_definition_conflict)
    #[cfg(feature = "term-tools")]
    pub async fn preview_merge(
        &self,
        source_id: &str,
        target_id: &str,
    ) -> Result<(i64, i64, bool)> {
        if source_id == target_id {
            anyhow::bail!("Cannot merge a term with itself");
        }

        // Verify both terms exist
        Term::find_by_id(&self.pool, source_id)
            .await
            .context("Source term not found")?;
        Term::find_by_id(&self.pool, target_id)
            .await
            .context("Target term not found")?;

        // Count variants
        let source_variants = TermVariant::find_by_term_id(&self.pool, source_id).await?;
        let variant_count = source_variants.len() as i64;

        // Count occurrences
        let occurrence_count = Occurrence::count_by_term_id(&self.pool, source_id).await?;

        // Check definition conflict
        let source_def = Definition::find_by_term_id(&self.pool, source_id).await?;
        let target_def = Definition::find_by_term_id(&self.pool, target_id).await?;
        let has_conflict = source_def.is_some() && target_def.is_some();

        Ok((variant_count, occurrence_count, has_conflict))
    }
}
