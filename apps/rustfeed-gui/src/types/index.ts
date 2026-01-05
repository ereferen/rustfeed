/** Feed information */
export interface Feed {
  id: number
  url: string
  title: string
  description: string | null
  created_at: string
  updated_at: string
}

/** Article information */
export interface Article {
  id: number
  feed_id: number
  title: string
  url: string
  content: string | null
  published_at: string | null
  is_read: boolean
  is_favorite: boolean
  created_at: string
}

/** Result of fetching all feeds */
export interface FetchResult {
  total_feeds: number
  new_articles: number
  errors: string[]
}
