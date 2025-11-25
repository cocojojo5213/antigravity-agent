import React, {useState} from 'react';
import {Calendar, Check, Clock, Copy, Key, Settings, User, X} from 'lucide-react';
import type {AntigravityAccount} from '@/commands/types/account.types';
import {BaseDialog, BaseDialogContent, BaseDialogHeader, BaseDialogTitle,} from '@/components/base-ui/BaseDialog';
import {BaseButton} from '@/components/base-ui/BaseButton';
import {cn} from '@/utils/utils';
import {logger} from '@/utils/logger';

interface BusinessUserDetailProps {
  isOpen: boolean;
  onOpenChange: (open: boolean) => void;
  user: AntigravityAccount | null;
}

const BusinessUserDetail: React.FC<BusinessUserDetailProps> = ({
  isOpen,
  onOpenChange,
  user
}) => {
  const [copiedField, setCopiedField] = useState<string | null>(null);

  // 复制到剪贴板功能
  const copyToClipboard = async (text: string, fieldName: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopiedField(fieldName);
      setTimeout(() => setCopiedField(null), 2000);
    } catch (error) {
      logger.error('复制失败', {
        module: 'UserDetail',
        action: 'copy_failed',
        error: error instanceof Error ? error.message : String(error)
      });
    }
  };

  // 格式化日期时间
  const formatDateTime = (dateString: string) => {
    try {
      const date = new Date(dateString);
      return date.toLocaleString('zh-CN', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit'
      });
    } catch (error) {
      return '无效日期';
    }
  };

  // 获取相对时间
  const getRelativeTime = (dateString: string) => {
    try {
      const date = new Date(dateString);
      const now = new Date();
      const diffMs = now.getTime() - date.getTime();
      const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
      const diffDays = Math.floor(diffHours / 24);

      if (diffDays > 0) {
        return `${diffDays}天前`;
      } else if (diffHours > 0) {
        return `${diffHours}小时前`;
      } else {
        return '刚刚';
      }
    } catch (error) {
      return '未知';
    }
  };

  // 解码 Base64 头像
  const getAvatarUrl = (base64Url: string) => {
    try {
      // 如果已经是完整URL，直接返回
      if (base64Url.startsWith('http') || base64Url.startsWith('data:')) {
        return base64Url;
      }
      // 如果是 Base64 编码，尝试解码
      return atob(base64Url);
    } catch (error) {
      // 解码失败，返回默认头像或空字符串
      return '';
    }
  };

  const InfoItem = ({
    icon,
    label,
    value,
    copyable = false,
    fieldName = '',
    isMultiline = false
  }: {
    icon: React.ReactNode;
    label: string;
    value: string;
    copyable?: boolean;
    fieldName?: string;
    isMultiline?: boolean;
  }) => (
    <div className="group">
      <label className="text-sm font-medium text-gray-900 dark:text-gray-100 mb-1.5 block px-1 flex items-center gap-2">
        {icon}
        <span>{label}</span>
      </label>
      <div className="relative">
        <div className={cn(
          "bg-gray-50 dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-md px-3 py-2 text-sm text-gray-600 dark:text-gray-400 break-all select-all transition-colors group-hover:border-gray-300 dark:group-hover:border-gray-700",
          isMultiline ? "min-h-[60px] whitespace-pre-wrap font-mono" : "font-mono"
        )}>
          {value || '未设置'}
        </div>
        {copyable && value && (
          <BaseButton
            variant="ghost"
            size="icon"
            className="absolute top-1 right-1 h-6 w-6 opacity-0 group-hover:opacity-100 transition-opacity"
            onClick={() => copyToClipboard(value, fieldName)}
            title="复制"
          >
            {copiedField === fieldName ? (
              <Check className="h-3.5 w-3.5 text-green-600" />
            ) : (
              <Copy className="h-3.5 w-3.5 text-gray-500" />
            )}
          </BaseButton>
        )}
      </div>
    </div>
  );

  if (!user) return null;

  const avatarUrl = getAvatarUrl(user.profile_url);

  return (
    <BaseDialog open={isOpen} onOpenChange={onOpenChange}>
      <BaseDialogContent className="max-w-lg p-0 overflow-hidden bg-white dark:bg-gray-950 border border-gray-100 dark:border-gray-800 shadow-2xl">
        <BaseDialogHeader className="px-5 py-4 border-b border-gray-100 dark:border-gray-800 bg-gray-50/50 dark:bg-gray-900/50">
          <div className="flex items-center justify-between">
            <BaseDialogTitle className="text-base font-semibold text-gray-900 dark:text-white flex items-center gap-2">
              <User className="h-4 w-4 text-gray-500" />
              <span>用户详情</span>
            </BaseDialogTitle>
            <BaseButton
              variant="ghost"
              size="icon"
              className="h-6 w-6 text-gray-400 hover:text-gray-600"
              onClick={() => onOpenChange(false)}
            >
              <X className="h-4 w-4" />
            </BaseButton>
          </div>
        </BaseDialogHeader>

        <div className="p-5 space-y-6 max-h-[70vh] overflow-y-auto">
          {/* 用户头像和基本信息 */}
          <div className="flex items-center gap-4 pb-4 border-b border-gray-100 dark:border-gray-800">
            {avatarUrl ? (
              <img
                src={avatarUrl}
                alt={user.name}
                className="h-16 w-16 rounded-full object-cover border-2 border-gray-100 dark:border-gray-800"
              />
            ) : (
              <div className="h-16 w-16 rounded-full bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center text-white text-xl font-semibold">
                {user.name.charAt(0).toUpperCase()}
              </div>
            )}
            <div className="flex-1">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
                {user.name}
              </h3>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                {user.email}
              </p>
            </div>
          </div>

          {/* 基本信息 */}
          <div className="space-y-4">
            <h4 className="text-xs font-semibold text-gray-400 uppercase tracking-wider px-1">基本信息</h4>

            <div className="space-y-3">
              <InfoItem
                icon={<User className="h-4 w-4 text-blue-500" />}
                label="账户名称"
                value={user.name}
                copyable
                fieldName="name"
              />

              <InfoItem
                icon={<User className="h-4 w-4 text-green-500" />}
                label="邮箱地址"
                value={user.email}
                copyable
                fieldName="email"
              />
            </div>
          </div>

          <div className="h-px bg-gray-100 dark:bg-gray-800" />

          {/* 技术信息 */}
          <div className="space-y-4">
            <h4 className="text-xs font-semibold text-gray-400 uppercase tracking-wider px-1">技术信息</h4>

            <div className="space-y-3">
              <InfoItem
                icon={<Key className="h-4 w-4 text-orange-500" />}
                label="API 密钥"
                value={user.api_key}
                copyable
                fieldName="api_key"
              />
            </div>
          </div>

          <div className="h-px bg-gray-100 dark:bg-gray-800" />

          {/* 时间信息 */}
          <div className="space-y-4">
            <h4 className="text-xs font-semibold text-gray-400 uppercase tracking-wider px-1">时间信息</h4>

            <div className="space-y-3">
              <InfoItem
                icon={<Calendar className="h-4 w-4 text-indigo-500" />}
                label="创建时间"
                value={`${formatDateTime(user.created_at)} (${getRelativeTime(user.created_at)})`}
                fieldName="created_at"
              />

              <InfoItem
                icon={<Clock className="h-4 w-4 text-cyan-500" />}
                label="最后切换"
                value={`${formatDateTime(user.last_switched)} (${getRelativeTime(user.last_switched)})`}
                fieldName="last_switched"
              />
            </div>
          </div>

          <div className="h-px bg-gray-100 dark:bg-gray-800" />

          {/* 用户设置 */}
          <div className="space-y-4">
            <h4 className="text-xs font-semibold text-gray-400 uppercase tracking-wider px-1">用户设置</h4>

            <InfoItem
              icon={<Settings className="h-4 w-4 text-gray-500" />}
              label="设置数据"
              value={user.user_settings}
              copyable
              fieldName="user_settings"
              isMultiline
            />
          </div>
        </div>
      </BaseDialogContent>
    </BaseDialog>
  );
};

export default BusinessUserDetail;
