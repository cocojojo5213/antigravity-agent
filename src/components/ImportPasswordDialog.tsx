import React, { useState } from 'react';
import * as Dialog from '@radix-ui/react-dialog';
import { X, Eye, EyeOff, Lock, Upload } from 'lucide-react';

interface ImportPasswordDialogProps {
  isOpen: boolean;
  onOpenChange: (open: boolean) => void;
  onSubmit: (password: string) => void;
  onCancel: () => void;
}

const ImportPasswordDialog: React.FC<ImportPasswordDialogProps> = ({
  isOpen,
  onOpenChange,
  onSubmit,
  onCancel
}) => {
  const [password, setPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [validationError, setValidationError] = useState<string>('');

  const validatePassword = (password: string) => {
    if (password.length < 4) return { isValid: false, message: '密码长度至少为4位' };
    if (password.length > 50) return { isValid: false, message: '密码长度不能超过50位' };
    return { isValid: true };
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    // 清除之前的错误
    setValidationError('');

    // 基本密码验证：确保密码不为空
    if (!password.trim()) {
      setValidationError('请输入密码');
      return;
    }

    // 自定义密码验证
    const validation = validatePassword(password);
    if (!validation.isValid) {
      setValidationError(validation.message || '密码无效');
      return;
    }

    onSubmit(password);
    // 重置状态
    setPassword('');
    setShowPassword(false);
    setValidationError('');
  };

  const handleClose = () => {
    // 重置状态
    setPassword('');
    setShowPassword(false);
    setValidationError('');
    onOpenChange(false);
  };

  const isValid = password.trim() !== '' && validatePassword(password).isValid;

  return (
    <Dialog.Root open={isOpen} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/50 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 z-50" />
        <Dialog.Content className="fixed left-[50%] top-[50%] translate-x-[-50%] translate-y-[-50%] w-full max-w-md bg-white dark:bg-gray-900 rounded-xl shadow-2xl data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[state=closed]:slide-out-to-left-1/2 data-[state=closed]:slide-out-to-top-[48%] data-[state=open]:slide-in-from-left-1/2 data-[state=open]:slide-in-from-top-[48%] z-50">
          <Dialog.Title className="text-lg font-semibold text-gray-900 dark:text-white p-6 pb-4 flex items-center gap-3">
            <Upload className="h-5 w-5 text-antigravity-blue" />
            导入配置文件
          </Dialog.Title>

          <Dialog.Description className="text-sm text-gray-600 dark:text-gray-400 px-6 pb-4">
            请输入配置文件的解密密码
          </Dialog.Description>

          <form onSubmit={handleSubmit} className="p-6 pt-0">
            <div className="space-y-4">
              {/* 密码输入 */}
              <div className="space-y-2">
                <label className="text-sm font-medium text-gray-700 dark:text-gray-300 flex items-center gap-2">
                  <Lock className="h-4 w-4" />
                  密码
                </label>
                <div className="relative">
                  <input
                    type={showPassword ? 'text' : 'password'}
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    placeholder="请输入密码"
                    className="w-full px-4 py-3 pr-12 bg-gray-50 dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-antigravity-blue focus:border-transparent transition-all duration-200 text-gray-900 dark:text-white placeholder-gray-500 dark:placeholder-gray-400"
                    autoFocus
                  />
                  <button
                    type="button"
                    onClick={() => setShowPassword(!showPassword)}
                    className="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors duration-200"
                  >
                    {showPassword ? (
                      <EyeOff className="h-4 w-4" />
                    ) : (
                      <Eye className="h-4 w-4" />
                    )}
                  </button>
                </div>
              </div>

              {/* 错误信息 */}
              {validationError && (
                <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-3">
                  <p className="text-sm text-red-600 dark:text-red-400">{validationError}</p>
                </div>
              )}

            </div>

            {/* 按钮组 */}
            <div className="flex gap-3 mt-6">
              <Dialog.Close asChild>
                <button
                  type="button"
                  onClick={handleClose}
                  className="flex-1 px-4 py-3 bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors duration-200 font-medium"
                >
                  取消
                </button>
              </Dialog.Close>
              <button
                type="submit"
                disabled={!isValid}
                className="flex-1 px-4 py-3 bg-antigravity-blue text-white rounded-lg hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed transition-all duration-200 font-medium shadow-sm hover:shadow-md"
              >
                确认
              </button>
            </div>
          </form>

          <Dialog.Close asChild>
            <button
              onClick={handleClose}
              className="absolute right-4 top-4 p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 rounded-full hover:bg-gray-100 dark:hover:bg-gray-800 transition-all duration-200"
              aria-label="关闭"
            >
              <X className="h-4 w-4" />
            </button>
          </Dialog.Close>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
};

export default ImportPasswordDialog;