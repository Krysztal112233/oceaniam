import addLinkRounded from "@iconify-icons/material-symbols/add-link-rounded";
import addRounded from "@iconify-icons/material-symbols/add-rounded";
import adminPanelSettingsOutlineRounded from "@iconify-icons/material-symbols/admin-panel-settings-outline-rounded";
import appsRounded from "@iconify-icons/material-symbols/apps-rounded";
import autorenewRounded from "@iconify-icons/material-symbols/autorenew-rounded";
import blockRounded from "@iconify-icons/material-symbols/block-rounded";
import brightnessAutoRounded from "@iconify-icons/material-symbols/brightness-auto-rounded";
import checkCircleOutlineRounded from "@iconify-icons/material-symbols/check-circle-outline-rounded";
import closeRounded from "@iconify-icons/material-symbols/close-rounded";
import contentCopyOutlineRounded from "@iconify-icons/material-symbols/content-copy-outline-rounded";
import darkModeOutlineRounded from "@iconify-icons/material-symbols/dark-mode-outline-rounded";
import dashboardOutlineRounded from "@iconify-icons/material-symbols/dashboard-outline-rounded";
import dataObjectRounded from "@iconify-icons/material-symbols/data-object-rounded";
import deleteOutlineRounded from "@iconify-icons/material-symbols/delete-outline-rounded";
import domainDisabledOutlineRounded from "@iconify-icons/material-symbols/domain-disabled-outline-rounded";
import domainRounded from "@iconify-icons/material-symbols/domain-rounded";
import editOutlineRounded from "@iconify-icons/material-symbols/edit-outline-rounded";
import errorOutlineRounded from "@iconify-icons/material-symbols/error-outline-rounded";
import filterListRounded from "@iconify-icons/material-symbols/filter-list-rounded";
import groupOffOutlineRounded from "@iconify-icons/material-symbols/group-off-outline-rounded";
import groupOutlineRounded from "@iconify-icons/material-symbols/group-outline-rounded";
import historyRounded from "@iconify-icons/material-symbols/history-rounded";
import inboxOutlineRounded from "@iconify-icons/material-symbols/inbox-outline-rounded";
import infoOutlineRounded from "@iconify-icons/material-symbols/info-outline-rounded";
import keyOutlineRounded from "@iconify-icons/material-symbols/key-outline-rounded";
import lightModeOutlineRounded from "@iconify-icons/material-symbols/light-mode-outline-rounded";
import lockOutlineRounded from "@iconify-icons/material-symbols/lock-outline-rounded";
import logoutRounded from "@iconify-icons/material-symbols/logout-rounded";
import manageSearchRounded from "@iconify-icons/material-symbols/manage-search-rounded";
import menuRounded from "@iconify-icons/material-symbols/menu-rounded";
import monitoringRounded from "@iconify-icons/material-symbols/monitoring-rounded";
import paletteOutlineRounded from "@iconify-icons/material-symbols/palette-outline-rounded";
import passwordRounded from "@iconify-icons/material-symbols/password-rounded";
import personAddOutlineRounded from "@iconify-icons/material-symbols/person-add-outline-rounded";
import personOutlineRounded from "@iconify-icons/material-symbols/person-outline-rounded";
import searchRounded from "@iconify-icons/material-symbols/search-rounded";
import settingsOutlineRounded from "@iconify-icons/material-symbols/settings-outline-rounded";
import signatureRounded from "@iconify-icons/material-symbols/signature-rounded";
import spaceDashboardOutlineRounded from "@iconify-icons/material-symbols/space-dashboard-outline-rounded";
import tokenOutlineRounded from "@iconify-icons/material-symbols/token-outline-rounded";
import translateRounded from "@iconify-icons/material-symbols/translate-rounded";
import tuneRounded from "@iconify-icons/material-symbols/tune-rounded";
import unfoldMoreRounded from "@iconify-icons/material-symbols/unfold-more-rounded";
import visibilityOutlineRounded from "@iconify-icons/material-symbols/visibility-outline-rounded";
import warningOutlineRounded from "@iconify-icons/material-symbols/warning-outline-rounded";
import { addIcon } from "@iconify/vue/offline";

const icons = {
  "add-link-rounded": addLinkRounded,
  "add-rounded": addRounded,
  "admin-panel-settings-outline-rounded": adminPanelSettingsOutlineRounded,
  "apps-rounded": appsRounded,
  "autorenew-rounded": autorenewRounded,
  "block-rounded": blockRounded,
  "brightness-auto-rounded": brightnessAutoRounded,
  "check-circle-outline-rounded": checkCircleOutlineRounded,
  "close-rounded": closeRounded,
  "content-copy-outline-rounded": contentCopyOutlineRounded,
  "dark-mode-outline-rounded": darkModeOutlineRounded,
  "dashboard-outline-rounded": dashboardOutlineRounded,
  "data-object-rounded": dataObjectRounded,
  "delete-outline-rounded": deleteOutlineRounded,
  "domain-disabled-outline-rounded": domainDisabledOutlineRounded,
  "domain-rounded": domainRounded,
  "edit-outline-rounded": editOutlineRounded,
  "error-outline-rounded": errorOutlineRounded,
  "filter-list-rounded": filterListRounded,
  "group-off-outline-rounded": groupOffOutlineRounded,
  "group-outline-rounded": groupOutlineRounded,
  "history-rounded": historyRounded,
  "inbox-outline-rounded": inboxOutlineRounded,
  "info-outline-rounded": infoOutlineRounded,
  "key-outline-rounded": keyOutlineRounded,
  "light-mode-outline-rounded": lightModeOutlineRounded,
  "lock-outline-rounded": lockOutlineRounded,
  "logout-rounded": logoutRounded,
  "manage-search-rounded": manageSearchRounded,
  "menu-rounded": menuRounded,
  "monitoring-rounded": monitoringRounded,
  "palette-outline-rounded": paletteOutlineRounded,
  "password-rounded": passwordRounded,
  "person-add-outline-rounded": personAddOutlineRounded,
  "person-outline-rounded": personOutlineRounded,
  "search-rounded": searchRounded,
  "settings-outline-rounded": settingsOutlineRounded,
  "signature-rounded": signatureRounded,
  "space-dashboard-outline-rounded": spaceDashboardOutlineRounded,
  "token-outline-rounded": tokenOutlineRounded,
  "translate-rounded": translateRounded,
  "tune-rounded": tuneRounded,
  "unfold-more-rounded": unfoldMoreRounded,
  "visibility-outline-rounded": visibilityOutlineRounded,
  "warning-outline-rounded": warningOutlineRounded,
} as const;

for (const [name, icon] of Object.entries(icons)) {
  addIcon(`material-symbols:${name}`, icon);
}
