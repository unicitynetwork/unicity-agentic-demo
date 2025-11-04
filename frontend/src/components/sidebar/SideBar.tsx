import React, { useState } from 'react'
import type { BalanceInfo } from '../../types';
import { AssetList } from './AssetList';
import { IdentityManager } from './IdentityManager';

interface SideBarProps {
  balances: BalanceInfo[];
}

const SideBar: React.FC<SideBarProps> = ({ balances }) => {
  const [activeTab, setActiveTab] = useState<'dashboard' | 'settings'>('dashboard');
  
  return (
    <div className='flex flex-col gap-2'>
      <div className="grid grid-cols-2 bg-[#171717] rounded-xl">
        <button
          onClick={() => setActiveTab('dashboard')}
          className={`
            py-1.5 rounded-xl text-sm font-medium transition-all
            ${activeTab === 'dashboard'
              ? 'bg-white text-brand-text-dark shadow-sm'
              : 'text-brand-text-dim hover:text-brand-text-light'
            }
          `}
        >
          Dashboard
        </button>
        <button
          onClick={() => setActiveTab('settings')}
          className={`
            py-1.5 rounded-xl text-sm font-medium transition-all
            ${activeTab === 'settings'
              ? 'bg-white text-brand-text-dark shadow-sm'
              : 'text-brand-text-dim hover:text-brand-text-light'
            }
          `}
        >
          Settings
        </button>
      </div>
      <IdentityManager/>
      <AssetList balances={balances}/>
    </div>
  )
}

export default SideBar