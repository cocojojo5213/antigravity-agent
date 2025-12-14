import {create} from "zustand";
import {AntigravityAccount} from "@/commands/types/account.types.ts";
import {CloudCodeAPI} from "@/services/cloudcode-api.ts";
import {CloudCodeAPITypes} from "@/services/cloudcode-api.types.ts";
import {AccountCommands} from "@/commands/AccountCommands.ts";
import {ProcessCommands} from "@/commands/ProcessCommands.ts";

type State = {
  data: Record<string, AccountAdditionData>
}

type Actions = {
  fetchData: (antigravityAccount: AntigravityAccount) => Promise<void>
}

// 暂时不知道 ultra 定义, 先模糊匹配,
export type UserTier = 'free-tier' | 'g1-pro-tier' | 'g1-ultra-tier';

export interface AccountAdditionData {
  geminiQuote: number
  claudeQuote: number
  userAvatar: string
  userId: string
}

export const useAccountAdditionData = create<State & Actions>((setState, getState) => ({
  data: {},
  fetchData: async (antigravityAccount: AntigravityAccount) => {
    let codeAssistResponse: CloudCodeAPITypes.LoadCodeAssistResponse | CloudCodeAPITypes.ErrorResponse = null

    try {
      codeAssistResponse = await CloudCodeAPI.loadCodeAssist(antigravityAccount.auth.access_token);
    } catch (e) {
      codeAssistResponse = e
    }

    // 如果存在错误, 则使用 ouath 重新获取 access token
    if ("error" in codeAssistResponse) {

      // 避免冲突 如果是当前账户, 并且 Antigravity 在运行, 则不刷新 access token
      const currentAccount = await AccountCommands.getCurrentAntigravityAccount()
      const isAntigravityRunning = await ProcessCommands.isRunning()
      if (antigravityAccount.context.email === currentAccount?.context.email && isAntigravityRunning) {
        return
      }
      try {
        // 刷新 access token
        const refreshTokenResponse = await CloudCodeAPI.refreshAccessToken(antigravityAccount.auth.id_token);
        // 更新一下内存里面的 access token, 这里就不写入本地了
        antigravityAccount.auth.access_token = refreshTokenResponse.access_token;
      } catch {
        // 刷新失败（例如缺少 OAuth client 配置/refresh token 无效），终止本次附加数据拉取
        return
      }
    }

    // 先模糊匹配下 tier 的定义, 因为我也知不道具体是啥
    if (antigravityAccount.context.plan.slug.includes("ultra")) {
      antigravityAccount.context.plan.slug = "g1-ultra-tier";
    }

    codeAssistResponse = await CloudCodeAPI.loadCodeAssist(antigravityAccount.auth.access_token);

    const modelsResponse = await CloudCodeAPI.fetchAvailableModels(antigravityAccount.auth.access_token, codeAssistResponse.cloudaicompanionProject);
    const userInfoResponse = await CloudCodeAPI.userinfo(antigravityAccount.auth.access_token);

    setState({
      data: {
        ...getState().data,
        [antigravityAccount.context.email]: {
          geminiQuote: modelsResponse.models["gemini-3-pro-high"].quotaInfo.remainingFraction,
          claudeQuote: modelsResponse.models["claude-opus-4-5-thinking"].quotaInfo.remainingFraction,
          userAvatar: userInfoResponse.picture,
          userId: userInfoResponse.id,
        }
      }
    })
  }
}))
