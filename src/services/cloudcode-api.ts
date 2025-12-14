import {fetch} from '@tauri-apps/plugin-http'
import {CloudCodeAPITypes} from "@/services/cloudcode-api.types.ts";

// HTTP 客户端配置
interface HTTPConfig {
  baseURL: string;
  headers: Record<string, string>;
}

const HTTP_CONFIG: HTTPConfig = {
  baseURL: 'https://daily-cloudcode-pa.sandbox.googleapis.com', // 默认使用沙盒环境
  headers: {
    "User-Agent": "antigravity/windows/amd64",
    "Content-Type": "application/json",
    "Accept": "application/json"
  }
};

const GOOGLE_OAUTH_TOKEN_ENDPOINT = 'https://oauth2.googleapis.com/token';

function getGoogleOAuthClientCredentials(): { clientId: string; clientSecret?: string } {
  const clientId = import.meta.env.VITE_GOOGLE_OAUTH_CLIENT_ID as string | undefined;
  const clientSecret = import.meta.env.VITE_GOOGLE_OAUTH_CLIENT_SECRET as string | undefined;

  if (!clientId) {
    throw new Error('Missing VITE_GOOGLE_OAUTH_CLIENT_ID (Google OAuth client_id)');
  }

  return {
    clientId,
    clientSecret: clientSecret || undefined,
  };
}


const post = async <T>(endpoint: string, data: any, options?: RequestInit): Promise<T> => {

  const requestConfig: RequestInit = {
    method: 'POST',
    body: JSON.stringify(data),
    ...options,
    headers: {
      ...HTTP_CONFIG.headers,
      ...(options?.headers || {})
    }
  };

  const response = await fetch(`${HTTP_CONFIG.baseURL}${endpoint}`, requestConfig);

  return await response.json();
}


// CloudCode API 服务命名空间
export namespace CloudCodeAPI {

  export async function fetchAvailableModels(
    authorizationKey: string,
    project: string,
  ): Promise<CloudCodeAPITypes.FetchAvailableModelsResponse> {

    const requestData = {
      "project": project
    };

    const response = await post<CloudCodeAPITypes.FetchAvailableModelsResponse | CloudCodeAPITypes.ErrorResponse>(
      '/v1internal:fetchAvailableModels',
      requestData,
      {
        headers: {
          'Authorization': `Bearer ${authorizationKey}`
        }
      }
    );

    if ("error" in response) {
      return Promise.reject(response);
    }

    return response;
  }

  export async function loadCodeAssist(
    authorizationKey: string,
  ) {
    const requestData = {metadata: {ideType: 'ANTIGRAVITY'}};

    const response = await post<CloudCodeAPITypes.LoadCodeAssistResponse | CloudCodeAPITypes.ErrorResponse>(
      '/v1internal:loadCodeAssist',
      requestData,
      {
        headers: {
          'Authorization': `Bearer ${authorizationKey}`
        }
      }
    )

    if ("error" in response) {
      return Promise.reject(response);
    }

    return response;
  }

  export async function refreshAccessToken(
    refresh_token: string,
  ) {
    const { clientId, clientSecret } = getGoogleOAuthClientCredentials();

    const body = new URLSearchParams({
      client_id: clientId,
      grant_type: 'refresh_token',
      refresh_token,
    });

    if (clientSecret) {
      body.set('client_secret', clientSecret);
    }

    const response = await fetch(
      GOOGLE_OAUTH_TOKEN_ENDPOINT,
      {
        method: 'POST',
        headers: {
          'content-type': 'application/x-www-form-urlencoded',
        },
        body: body.toString(),
      },
    );
    const json = await response.json() as unknown as CloudCodeAPITypes.RefreshAccessTokenResponse | CloudCodeAPITypes.ErrorResponse;

    if ("error" in json) {
      return Promise.reject(json);
    }

    return json;
  }

  export async function userinfo(
    access_token: string,
  ) {

    const response = await fetch(
      'https://www.googleapis.com/oauth2/v2/userinfo',
      {
        headers: {
          'Authorization': `Bearer ${access_token}`
        }
      },
    );
    const json = await response.json() as unknown as CloudCodeAPITypes.UserInfoResponse | CloudCodeAPITypes.ErrorResponse;

    if ("error" in json) {
      return Promise.reject(json);
    }

    return json;
  }

}

