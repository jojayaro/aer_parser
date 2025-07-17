(
  select * from 'CSV/WELLS*.csv'
  except
  select * from delta_scan('./st1')
)
union all
(
  select * from delta_scan('./st1')
  except
  select * from 'CSV/WELLS*.csv'
);