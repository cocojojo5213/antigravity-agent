export namespace LanguageServerResponse {
  export interface Root {
    userStatus: UserStatus
  }

  export interface UserStatus {
    acceptedLatestTermsOfService: boolean
    cascadeModelConfigData: CascadeModelConfigData
    disableTelemetry: boolean
    email: string
    name: string
    planStatus: PlanStatus
  }

  export interface CascadeModelConfigData {
    clientModelConfigs: ClientModelConfig[]
    clientModelSorts: ClientModelSort[]
    defaultOverrideModelConfig: DefaultOverrideModelConfig
  }

  export interface ClientModelConfig {
    allowedTiers: string[]
    isRecommended: boolean
    label: string
    modelOrAlias: ModelOrAlias
    quotaInfo: QuotaInfo
    supportsImages?: boolean
  }

  export interface ModelOrAlias {
    model: string
  }

  export interface QuotaInfo {
    remainingFraction: number
    resetTime: string
  }

  export interface ClientModelSort {
    groups: Group[]
    name: string
  }

  export interface Group {
    modelLabels: string[]
  }

  export interface DefaultOverrideModelConfig {
    modelOrAlias: ModelOrAlias2
  }

  export interface ModelOrAlias2 {
    model: string
  }

  export interface PlanStatus {
    availableFlowCredits: number
    availablePromptCredits: number
    planInfo: PlanInfo
  }

  export interface PlanInfo {
    allowPremiumCommandModels: boolean
    allowStickyPremiumModels: boolean
    browserEnabled: boolean
    canAllowCascadeInBackground: boolean
    canBuyMoreCredits: boolean
    canCustomizeAppIcon: boolean
    canGenerateCommitMessages: boolean
    cascadeCanAutoRunCommands: boolean
    cascadeWebSearchEnabled: boolean
    defaultTeamConfig: DefaultTeamConfig
    hasAutocompleteFastMode: boolean
    hasTabToJump: boolean
    knowledgeBaseEnabled: boolean
    maxCustomChatInstructionCharacters: string
    maxLocalIndexSize: string
    maxNumChatInputTokens: string
    maxNumPinnedContextItems: string
    maxNumPremiumChatMessages: string
    monthlyFlexCreditPurchaseAmount: number
    monthlyFlowCredits: number
    monthlyPromptCredits: number
    planName: string
    teamsTier: string
  }

  export interface DefaultTeamConfig {
    allowAutoRunCommands: boolean
    allowBrowserExperimentalFeatures: boolean
    allowMcpServers: boolean
  }
}
