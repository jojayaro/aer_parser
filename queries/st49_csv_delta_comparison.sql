(
  select * from 'CSV/SPUD*.csv'
  except
  select * from delta_scan('./st49')
)
union all
(
  select * from delta_scan('./st49')
  except
  select * from 'CSV/SPUD*.csv'
);