import axios, { AxiosInstance } from 'axios'
import { QueryClient } from '@tanstack/react-query'

// API client using axios and react-query
const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || '/api'

class ApiClient {
  private client: AxiosInstance

  constructor(baseURL: string) {
    this.client = axios.create({
      baseURL,
      headers: {
        'Content-Type': 'application/json',
      },
      timeout: 30000, // 30 seconds
    })

    // Response interceptor for error handling
    this.client.interceptors.response.use(
      (response) => response,
      (error) => {
        console.error('API Error:', error.response?.data || error.message)
        return Promise.reject(error)
      }
    )
  }

  // Papers API
  async importPaperFile(file: File, title?: string) {
    const formData = new FormData()
    formData.append('file', file)
    if (title) {
      formData.append('title', title)
    }
    return this.client.post('/papers/import', formData, {
      headers: { 'Content-Type': 'multipart/form-data' },
    })
  }

  async importPaperUrl(url: string, title: string) {
    const formData = new FormData()
    formData.append('url', url)
    formData.append('title', title)
    return this.client.post('/papers/import', formData, {
      headers: { 'Content-Type': 'multipart/form-data' },
    })
  }

  async listPapers(page = 1, limit = 20, status?: string) {
    return this.client.get('/papers', {
      params: { page, limit, status },
    })
  }

  async getPaper(id: string) {
    return this.client.get(`/papers/${id}`)
  }

  async getTranslation(paperId: string) {
    return this.client.get(`/papers/${paperId}/translation`)
  }

  async processPaper(paperId: string) {
    return this.client.post(`/papers/${paperId}/process`)
  }

  async getPaperStatus(paperId: string) {
    return this.client.get(`/papers/${paperId}/status`)
  }

  async retryChunk(chunkId: string) {
    return this.client.post(`/chunks/${chunkId}/retry`)
  }

  // Terms API
  async searchTerms(query?: string, lang = 'both', sort = 'alphabetical', page = 1, limit = 50) {
    return this.client.get('/terms', {
      params: { q: query, lang, sort, page, limit },
    })
  }

  async getTerm(id: string) {
    return this.client.get(`/terms/${id}`)
  }

  async createTerm(data: {
    lemma_en: string
    lemma_ja: string
    reading_kana?: string
    pos?: string
    tags?: string
    note?: string
  }) {
    return this.client.post('/terms', data)
  }

  async updateTerm(id: string, data: Partial<{
    lemma_en: string
    lemma_ja: string
    reading_kana: string
    pos: string
    tags: string
    note: string
  }>) {
    return this.client.patch(`/terms/${id}`, data)
  }

  async deleteTerm(id: string) {
    return this.client.delete(`/terms/${id}`)
  }

  async mergeTerms(sourceId: string, targetId: string) {
    return this.client.post('/terms/merge', {
      source_id: sourceId,
      target_id: targetId,
      confirmed: true,
    })
  }

  async generateDefinition(termId: string) {
    return this.client.post(`/terms/${termId}/define`)
  }

  async findDuplicates() {
    return this.client.get('/terms/duplicates')
  }

  // Occurrences API
  async listOccurrences(paperId?: string, termId?: string) {
    return this.client.get('/occurrences', {
      params: { paper_id: paperId, term_id: termId },
    })
  }
}

export const apiClient = new ApiClient(API_BASE_URL)

// React Query client configuration
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60 * 5, // 5 minutes
      retry: 1,
      refetchOnWindowFocus: false,
    },
  },
})
