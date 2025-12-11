import {create} from "zustand";
import {AntigravityAccount} from "@/commands/types/account.types.ts";
import {CloudCodeAPI} from "@/services/cloudcode-api.ts";
import {CloudCodeAPITypes} from "@/services/cloudcode-api.types.ts";
import {AccountCommands} from "@/commands/AccountCommands.ts";

type State = {
  data: Record<string, CloudCodeAPITypes.FetchAvailableModelsResponse>
}

type Actions = {
  fetchData: (antigravityAccount: AntigravityAccount) => Promise<void>
}

export const useAvailableModels = create<State & Actions>((setState, getState) => ({
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

      // 这里一定不是当前账户, 但是为了保险起见, 还是检查一下
      const currentAccount = await AccountCommands.getCurrentAntigravityAccount()
      if (antigravityAccount.context.email === currentAccount?.context.email) {
        return
      }
      // 刷新 access token
      const refreshTokenResponse = await CloudCodeAPI.refreshAccessToken(antigravityAccount.auth.id_token);
      // 更新一下内存里面的 access token, 这里就不写入本地了
      antigravityAccount.auth.access_token = refreshTokenResponse.access_token;
    }

    codeAssistResponse = await CloudCodeAPI.loadCodeAssist(antigravityAccount.auth.access_token);

    const modelsResponse = await CloudCodeAPI.fetchAvailableModels(antigravityAccount.auth.access_token, codeAssistResponse.cloudaicompanionProject);

    setState({
      data: {
        ...getState().data,
        [antigravityAccount.context.email]: modelsResponse
      }
    })
  }
}))
