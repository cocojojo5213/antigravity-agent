import React, {useState} from 'react';
import {Check, Copy, Key, User} from 'lucide-react';
import type {AntigravityAccount} from '@/commands/types/account.types';
import {BaseButton} from '@/components/base-ui/BaseButton';
import {cn} from '@/utils/utils';
import {logger} from '@/utils/logger';
import {Modal} from "antd";

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
      await navigator.clipboard.writeText(user[fieldName]);
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
    copyText?: string;
    fieldName?: string;
    isMultiline?: boolean;
  }) => (
    <div className="group">
      <label className="text-sm font-medium text-gray-900 dark:text-gray-100 mb-1.5 px-1 flex items-center gap-2">
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

  return (
    <Modal
      footer={null}
      open={isOpen}
      onCancel={() => onOpenChange(false)}
    >
      {<div className={"flex flex-row items-center gap-0.5"}>
        <User className="h-4 w-4 text-gray-500"/>
        <span>用户详情</span>
      </div>}
      <div className="p-5 space-y-6 max-h-[70vh] overflow-y-auto">
        {/* 用户头像和基本信息 */}
        <div className="flex items-center gap-4">
          <img
            src={""}
            alt={user.context.plan_name}
            className="h-16 w-16 rounded-full object-cover border-2 border-gray-100 dark:border-gray-800"
          />
          <div className="flex-1">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
              {user.context.plan_name}
            </h3>
            <p className="text-sm text-gray-500 dark:text-gray-400">
              {user.context.email}
            </p>
          </div>
        </div>

        <div className="h-px bg-gray-100 dark:bg-gray-800"/>

        <InfoItem
          icon={<Key className="h-4 w-4 text-orange-500"/>}
          label="API 密钥"
          value={"****"}
          copyable
          fieldName="api_key"
        />
      </div>
    </Modal>
  );
};

export default BusinessUserDetail;
