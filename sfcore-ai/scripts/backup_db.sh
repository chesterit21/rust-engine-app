#!/bin/bash
# Helper script to backup database
cp /home/sfcore/server-db/SFCoreProTM.db /home/sfcore/server-db/SFCoreProTM_backup_$(date +%Y%m%d_%H%M%S).db
echo "Database backed up."
